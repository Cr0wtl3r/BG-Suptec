<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    ObterInformacoesSistema,
    AlterarIP,
    AlterarNomeComputador, // <-- Retorna (bool, error) agora
    AlterarDNS,
    ReiniciarComputador, // <-- Importar
  } from "../../../wailsjs/go/main/App";
  import Accordion from "../shared/Accordion.svelte";
  import EditableField from "../shared/EditableField.svelte";
  import Modal from "../shared/Modal.svelte"; // 1. IMPORTAR O NOVO MODAL

  // ... (toda a lógica do script até as funções de salvar) ...
  let info: InfoSistema | null = null;
  let erro = "";
  let campoBuscaElement: HTMLInputElement;
  let gridContainerElement: HTMLDivElement;
  let colunasDoGrid = 3;
  let linhasDoGrid = 4;
  let itensPorPagina = 12;

  onMount(() => {
    carregarInformacoes();
    window.addEventListener("keydown", handleGlobalKeydown);
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        const LARGURA_MINIMA_ITEM = 180;
        const GAP_DO_GRID = 12;
        const ALTURA_MINIMA_ITEM = 70;
        const novasColunas = Math.max(
          1,
          Math.floor(width / (LARGURA_MINIMA_ITEM + GAP_DO_GRID)),
        );
        const novasLinhas = Math.max(
          1,
          Math.floor(height / (ALTURA_MINIMA_ITEM + GAP_DO_GRID)),
        );
        colunasDoGrid = novasColunas;
        linhasDoGrid = novasLinhas;
        itensPorPagina = colunasDoGrid * linhasDoGrid;
      }
    });
    if (gridContainerElement) {
      observer.observe(gridContainerElement);
    }
    return () => {
      window.removeEventListener("keydown", handleGlobalKeydown);
      if (gridContainerElement) {
        observer.unobserve(gridContainerElement);
      }
    };
  });

  type Modulo = {
    nome: string;
    funcionalidades: string[];
  };
  export let modulos: Modulo[] = [];
  const dispatch = createEventDispatcher();
  type InfoSistema = {
    nomeComputador: string;
    versaoWindows: string;
    edicaoWindows: string;
    buildWindows: string;
    processador: string;
    memoriaTotalGB: string;
    enderecoMAC: string;
    enderecoIP: string;
    mascaraRede: string;
    gatewayPadrao: string;
    dnsPrimario: string;
    dnsSecundario: string;
    interfaceAtiva: string;
  };
  type ModalInfo = {
    visible: boolean;
    tipo?: "sucesso" | "erro" | "aviso";
    titulo?: string;
    mensagem?: string;
    textoConfirmar?: string | null;
    textoCancelar?: string;
    onConfirm?: () => void;
  };
  let editandoIP = false;
  let editandoDNS = false;
  let tempEnderecoIP = "";
  let tempMascaraRede = "";
  let tempGatewayPadrao = "";
  let tempDNSPrimario = "";
  let tempDNSSecundario = "";
  let salvandoNome = false;
  let salvandoIP = false;
  let salvandoDNS = false;
  let modal: ModalInfo = { visible: false };
  let isInfoSistemaOpen = false;
  let isInfoRedeOpen = false;
  let isInfoDNSOpen = false;
  let termoBusca = "";
  let paginaAtual = 1;
  $: todasFuncionalidades = modulos
    .flatMap((m) => m.funcionalidades)
    .filter((f) => f !== "Painel de Informações")
    .sort((a, b) => a.localeCompare(b));
  $: funcionalidadesFiltradas = todasFuncionalidades.filter((f) =>
    f.toLowerCase().includes(termoBusca.toLowerCase()),
  );
  $: totalPaginas = Math.ceil(funcionalidadesFiltradas.length / itensPorPagina);
  $: funcionalidadesPaginadas = funcionalidadesFiltradas.slice(
    (paginaAtual - 1) * itensPorPagina,
    paginaAtual * itensPorPagina,
  );
  $: {
    if (termoBusca !== undefined) {
      paginaAtual = 1;
    }
  }
  const handleGlobalKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.altKey || event.metaKey || event.key.length > 1)
      return;
    if (document.activeElement?.tagName.toLowerCase() !== "input") {
      campoBuscaElement?.focus();
    }
  };
  function mostrarModal(info: Omit<ModalInfo, "visible">) {
    modal = { ...info, visible: true };
  }
  function fecharModal() {
    modal = { visible: false };
  }

  async function carregarInformacoes() {
    try {
      info = await ObterInformacoesSistema();
      erro = "";
    } catch (e) {
      erro = `Erro ao carregar informações: ${e}`;
      info = null;
    }
  }
  function iniciarEdicao(tipo: "ip" | "dns") {
    editandoIP = tipo === "ip";
    editandoDNS = tipo === "dns";
    if (tipo === "ip") {
      tempEnderecoIP = info?.enderecoIP || "";
      tempMascaraRede = info?.mascaraRede || "255.255.255.0";
      tempGatewayPadrao = info?.gatewayPadrao || "";
    }
    if (tipo === "dns") {
      if (info && info.dnsPrimario !== undefined) {
        tempDNSPrimario = info.dnsPrimario !== "N/A" ? info.dnsPrimario : "";
      } else {
        tempDNSPrimario = "";
      }
      if (info && info.dnsSecundario !== undefined) {
        tempDNSSecundario =
          info.dnsSecundario !== "N/A" ? info.dnsSecundario : "";
      } else {
        tempDNSSecundario = "";
      }
    }
  }
  async function handleSalvarNome(event: CustomEvent) {
    const novoNome = event.detail.trim();
    if (!novoNome) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Validação",
        mensagem: "Nome do computador não pode estar vazio!",
      });
      return;
    }
    salvandoNome = true;
    try {
      const precisaReiniciar = await AlterarNomeComputador(novoNome);
      await carregarInformacoes();

      if (precisaReiniciar) {
        mostrarModal({
          tipo: "sucesso",
          titulo: "Sucesso!",
          mensagem:
            "O nome do computador foi alterado. É necessário reiniciar para que a mudança tenha efeito completo. Deseja reiniciar agora?",
          textoConfirmar: "Reiniciar Agora",
          textoCancelar: "Depois",
          onConfirm: async () => {
            try {
              await ReiniciarComputador();
            } catch (err) {
              mostrarModal({
                tipo: "erro",
                titulo: "Erro",
                mensagem: `Não foi possível reiniciar: ${err}`,
              });
            }
            fecharModal();
          },
        });
      } else {
        mostrarModal({
          tipo: "sucesso",
          titulo: "Sucesso!",
          mensagem: "Nome do computador alterado!",
        });
      }
    } catch (e) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro",
        mensagem: `Erro ao alterar nome: ${e}`,
      });
    }
    salvandoNome = false;
  }

  async function salvarIP() {
    const ip = tempEnderecoIP.trim();
    const mascara = tempMascaraRede.trim();
    const gateway = tempGatewayPadrao.trim();

    if (ip !== "" && (mascara === "" || gateway === "")) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Validação",
        mensagem:
          "Para definir um IP estático, todos os campos (IP, Máscara e Gateway) devem ser preenchidos.",
      });
      return;
    }

    if (!info?.interfaceAtiva) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Sistema",
        mensagem: "Interface de rede não encontrada. Tente recarregar.",
      });
      return;
    }

    salvandoIP = true;
    try {
      await AlterarIP(info.interfaceAtiva, ip, mascara, gateway);
      const mensagem =
        ip === ""
          ? "Configurações de rede definidas para DHCP (dinâmico)!"
          : "Configurações de rede alteradas com sucesso!";

      mostrarModal({ tipo: "sucesso", titulo: "Sucesso!", mensagem: mensagem });

      await new Promise((resolve) => setTimeout(resolve, 2000));
      await carregarInformacoes();
      editandoIP = false;
    } catch (e) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro",
        mensagem: `Erro ao alterar IP: ${e}`,
      });
    }
    salvandoIP = false;
  }

  // MUDANÇA: Lógica de validação ajustada
  async function salvarDNS() {
    const primario = tempDNSPrimario.trim();
    const secundario = tempDNSSecundario.trim();

    if (!info?.interfaceAtiva) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Sistema",
        mensagem: "Interface de rede não encontrada. Tente recarregar.",
      });
      return;
    }

    salvandoDNS = true;
    try {
      await AlterarDNS(info.interfaceAtiva, primario, secundario);
      const mensagem =
        primario === ""
          ? "Servidores DNS definidos para obter via DHCP (automático)!"
          : "Servidores DNS alterados com sucesso!";

      mostrarModal({ tipo: "sucesso", titulo: "Sucesso!", mensagem: mensagem });

      await new Promise((resolve) => setTimeout(resolve, 1500));
      await carregarInformacoes();
      editandoDNS = false;
    } catch (e) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro",
        mensagem: `Erro ao alterar DNS: ${e}`,
      });
    }
    salvandoDNS = false;
  }

  function cancelarEdicao() {
    editandoIP = false;
    editandoDNS = false;
  }
  function navegarPara(funcionalidade: string) {
    dispatch("navigate", funcionalidade);
  }
</script>

<div
  class="w-full h-full bg-primary-purple bg-opacity-10 backdrop-blur-md border border-gray-700 rounded-xl animate-fadeIn flex flex-col md:flex-row gap-5 overflow-hidden"
>
  <div
    class="md:flex-shrink-0 w-full md:w-96 p-5 rounded-lg overflow-y-auto bg-dark-blue-light bg-opacity-25"
  >
    <h2
      class="text-2xl font-bold text-center text-accent-orange mb-4 bg-opacity-35"
    >
      Painel de Informações
    </h2>
    {#if erro}
      <p class="text-red-500 text-center opacity-80">{erro}</p>
    {:else if !info}
      <p class="text-center opacity-80">Carregando...</p>
    {:else}
      <div class="grid grid-cols-1 gap-3">
        <Accordion
          title="Informações do Sistema"
          bind:isOpen={isInfoSistemaOpen}
        >
          <div class="grid grid-cols-1 gap-3">
            <EditableField
              label="Computador"
              value={info.nomeComputador}
              disabled={salvandoNome}
              on:save={handleSalvarNome}
            />
            <div
              class="bg-dark-blue-bg p-3 rounded-md border-l-4 text-center border-primary-purple"
            >
              <span class="block text-xs opacity-70 uppercase mb-1">RAM</span>
              <span class="text-sm font-semibold break-all leading-snug"
                >{info.memoriaTotalGB}</span
              >
            </div>
            <div
              class="bg-dark-blue-bg p-3 rounded-md border-l-4 text-center border-primary-purple col-span-1"
            >
              <span class="block text-xs opacity-70 uppercase mb-1"
                >Windows</span
              >
              <span class="text-sm font-semibold break-all leading-snug"
                >{info.edicaoWindows} ({info.versaoWindows})</span
              >
            </div>
            <div
              class="bg-dark-blue-bg p-3 rounded-md text-center border-l-4 border-primary-purple col-span-1"
            >
              <span class="block text-xs opacity-70 uppercase mb-1"
                >Processador</span
              >
              <span class="text-sm font-semibold break-all leading-snug"
                >{info.processador}</span
              >
            </div>
            <div
              class="bg-dark-blue-bg p-3 text-center rounded-md border-l-4 border-primary-purple"
            >
              <span class="block text-xs opacity-70 uppercase mb-1">MAC</span>
              <span class="text-sm font-semibold break-all leading-snug"
                >{info.enderecoMAC}</span
              >
            </div>
          </div>
        </Accordion>
        <Accordion
          title="Informações de Rede (IPv4)"
          bind:isOpen={isInfoRedeOpen}
        >
          <div class="grid grid-cols-1 gap-3">
            <div
              class="bg-dark-blue-bg text-center p-3 rounded-md border-l-4 border-primary-purple col-span-1"
            >
              <span class="block text-xs opacity-70 uppercase mb-1"
                >Rede IPv4</span
              >
              <div class="flex items-start gap-2 min-h-6 w-full relative">
                {#if editandoIP}
                  <div class="flex flex-col gap-1 w-full">
                    <input
                      type="text"
                      bind:value={tempEnderecoIP}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border text-center"
                      disabled={salvandoIP}
                      placeholder="IP (deixe em branco para DHCP)"
                    />
                    <input
                      type="text"
                      bind:value={tempMascaraRede}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border text-center"
                      disabled={salvandoIP}
                      placeholder="Máscara de Sub-rede"
                    />
                    <input
                      type="text"
                      bind:value={tempGatewayPadrao}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border text-center"
                      disabled={salvandoIP}
                      placeholder="Gateway Padrão"
                    />
                    <div class="flex items-center gap-2 justify-end mt-1">
                      <button
                        aria-label="Salvar IP"
                        on:click={salvarIP}
                        disabled={salvandoIP}
                        title="Salvar"
                        class="bg-transparent border-none cursor-pointer text-text-light opacity-60 transition-opacity duration-200 p-0.5 hover:opacity-100 hover:text-green-500 disabled:cursor-not-allowed disabled:opacity-20"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="16"
                          height="16"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12"></polyline></svg
                        >
                      </button>
                      <button
                        aria-label="Cancelar edição de IP"
                        on:click={cancelarEdicao}
                        disabled={salvandoIP}
                        title="Cancelar"
                        class="bg-transparent border-none cursor-pointer text-text-light opacity-60 transition-opacity duration-200 p-0.5 hover:opacity-100 hover:text-red-500 disabled:cursor-not-allowed disabled:opacity-20"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="16"
                          height="16"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><line x1="18" y1="6" x2="6" y2="18"></line><line
                            x1="6"
                            y1="6"
                            x2="18"
                            y2="18"
                          ></line></svg
                        >
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="flex items-center justify-center w-full relative">
                    <span
                      class="text-sm font-semibold break-all leading-snug text-center w-full"
                    >
                      IP: {info.enderecoIP}<br />Máscara: {info.mascaraRede}<br
                      />Gateway: {info.gatewayPadrao}
                    </span>
                    <button
                      on:click={() => iniciarEdicao("ip")}
                      class="absolute right-0 top-1/2 -translate-y-1/2 p-0.5 bg-transparent text-text-light opacity-60 transition-all duration-200 hover:!opacity-100 hover:text-accent-orange rounded-md"
                      aria-label="Editar configurações de rede"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><path
                          d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"
                        ></path></svg
                      >
                    </button>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </Accordion>
        <Accordion title="Configurações de DNS" bind:isOpen={isInfoDNSOpen}>
          <div class="grid grid-cols-1 gap-3">
            <div
              class="bg-dark-blue-bg p-3 rounded-md border-l-4 text-center border-primary-purple col-span-1"
            >
              <span class="block text-xs opacity-70 uppercase mb-1">DNS</span>
              <div class="flex items-start gap-2 min-h-6 w-full relative">
                {#if editandoDNS}
                  <div class="flex flex-col gap-1 w-full">
                    <input
                      type="text"
                      bind:value={tempDNSPrimario}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border text-center"
                      disabled={salvandoDNS}
                      placeholder="DNS (deixe em branco para auto)"
                    />
                    <input
                      type="text"
                      bind:value={tempDNSSecundario}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border text-center"
                      disabled={salvandoDNS}
                      placeholder="DNS Secundário (opcional)"
                    />
                    <div class="flex items-center gap-2 justify-end mt-1">
                      <button
                        aria-label="Salvar DNS"
                        on:click={salvarDNS}
                        disabled={salvandoDNS}
                        title="Salvar"
                        class="bg-transparent border-none cursor-pointer text-text-light opacity-60 transition-opacity duration-200 p-0.5 hover:opacity-100 hover:text-green-500 disabled:cursor-not-allowed disabled:opacity-20"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="16"
                          height="16"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12"></polyline></svg
                        >
                      </button>
                      <button
                        aria-label="Cancelar edição de DNS"
                        on:click={cancelarEdicao}
                        disabled={salvandoDNS}
                        title="Cancelar"
                        class="bg-transparent border-none cursor-pointer text-text-light opacity-60 transition-opacity duration-200 p-0.5 hover:opacity-100 hover:text-red-500 disabled:cursor-not-allowed disabled:opacity-20"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="16"
                          height="16"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><line x1="18" y1="6" x2="6" y2="18"></line><line
                            x1="6"
                            y1="6"
                            x2="18"
                            y2="18"
                          ></line></svg
                        >
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="flex items-center justify-center w-full relative">
                    <span
                      class="text-sm font-semibold break-all leading-snug text-center w-full"
                    >
                      {#if info.dnsPrimario && info.dnsPrimario !== "N/A"}
                        Primário: {info.dnsPrimario}
                        {#if info.dnsSecundario && info.dnsSecundario !== "N/A"}
                          <br />Secundário: {info.dnsSecundario}
                        {/if}
                      {:else}
                        Não configurado ou Automático
                      {/if}
                    </span>
                    <button
                      on:click={() => iniciarEdicao("dns")}
                      class="absolute right-0 top-1/2 -translate-y-1/2 p-0.5 bg-transparent text-text-light opacity-60 transition-all duration-200 hover:!opacity-100 hover:text-accent-orange rounded-md"
                      aria-label="Editar servidores DNS"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><path
                          d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"
                        ></path></svg
                      >
                    </button>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </Accordion>
      </div>
    {/if}
  </div>

  <div class="flex-grow flex flex-col min-w-0 p-5">
    <h3 class="text-xl font-bold text-accent-orange mb-5">
      Todas as Ferramentas ({funcionalidadesFiltradas.length})
    </h3>
    <input
      type="search"
      class="flex-shrink-0 w-full box-border p-2.5 mb-4 bg-dark-blue-bg bg-opacity-70 border border-primary-purple text-text-light rounded-lg text-base focus:outline-none focus:ring-1 focus:ring-primary-purple"
      placeholder="Pesquisar ferramenta..."
      bind:value={termoBusca}
      bind:this={campoBuscaElement}
    />
    <div
      class="flex-grow overflow-y-auto pr-2 grid gap-3 pb-3 content-start"
      bind:this={gridContainerElement}
      style="grid-template-columns: repeat({colunasDoGrid}, minmax(0, 1fr));"
    >
      {#each funcionalidadesPaginadas as funcionalidade (funcionalidade)}
        <button
          class="p-4 border bg-opacity-50 text-center border-dark-blue-bg bg-dark-blue-light text-text-light rounded-lg cursor-pointer text-sm font-bold transition-all duration-200 hover:bg-primary-purple hover:translate-y-px hover:border-accent-orange"
          on:click={() => navegarPara(funcionalidade)}
        >
          {funcionalidade}
        </button>
      {/each}
      {#if funcionalidadesPaginadas.length === 0 && funcionalidadesFiltradas.length > 0}
        <p class="opacity-80 text-center col-span-full">
          Nenhuma ferramenta encontrada nesta página.
        </p>
      {:else if funcionalidadesFiltradas.length === 0}
        <p class="opacity-80 text-center col-span-full">
          Nenhuma ferramenta encontrada.
        </p>
      {/if}
    </div>
    {#if funcionalidadesFiltradas.length > itensPorPagina}
      <div class="flex-shrink-0 flex justify-center items-center gap-2 mt-4">
        <button
          class="px-3 py-1 bg-primary-purple text-white rounded-md cursor-pointer hover:bg-primary-purple-dark disabled:opacity-50 disabled:cursor-not-allowed"
          on:click={() => (paginaAtual = Math.max(1, paginaAtual - 1))}
          disabled={paginaAtual === 1}
        >
          Anterior
        </button>
        <span class="text-text-light text-sm"
          >Página {paginaAtual} de {totalPaginas}</span
        >
        <button
          class="px-3 py-1 bg-primary-purple text-white rounded-md cursor-pointer hover:bg-primary-purple-dark disabled:opacity-50 disabled:cursor-not-allowed"
          on:click={() =>
            (paginaAtual = Math.min(totalPaginas, paginaAtual + 1))}
          disabled={paginaAtual === totalPaginas}
        >
          Próxima
        </button>
      </div>
    {/if}
  </div>
</div>
{#if modal.visible}
  <Modal
    tipo={modal.tipo}
    titulo={modal.titulo || ""}
    mensagem={modal.mensagem || ""}
    textoConfirmar={modal.textoConfirmar}
    textoCancelar={modal.textoCancelar}
    on:confirmar={() => {
      if (modal.onConfirm) modal.onConfirm();
    }}
    on:cancelar={fecharModal}
  />
{/if}

<style>
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .animate-fadeIn {
    animation: fadeIn 0.5s ease-out;
  }

  @keyframes modalFadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes modalSlideIn {
    from {
      opacity: 0;
      transform: translateY(-20px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>

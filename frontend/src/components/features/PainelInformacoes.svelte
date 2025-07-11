<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    ObterInformacoesSistema,
    AlterarIP,
    AlterarNomeComputador,
    AlterarDNS,
  } from "../../../wailsjs/go/main/App";
  import Accordion from "../shared/Accordion.svelte";

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

  type ModalData = {
    tipo: "sucesso" | "erro";
    titulo: string;
    mensagem: string;
    visible: boolean;
  };

  let info: InfoSistema | null = null;
  let erro = "";
  let campoBuscaElement: HTMLInputElement;

  let editandoNome = false;
  let editandoIP = false;
  let editandoDNS = false;

  let tempNomeComputador = "";
  let tempEnderecoIP = "";
  let tempMascaraRede = "";
  let tempGatewayPadrao = "";
  let tempDNSPrimario = "";
  let tempDNSSecundario = "";

  let salvandoNome = false;
  let salvandoIP = false;
  let salvandoDNS = false;

  let modal: ModalData = {
    tipo: "sucesso",
    titulo: "",
    mensagem: "",
    visible: false,
  };

  let isInfoSistemaOpen = false;
  let isInfoRedeOpen = false;
  let isInfoDNSOpen = false;

  let termoBusca = "";

  const itensPorPagina = 12;
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

  onMount(() => {
    carregarInformacoes();
    window.addEventListener("keydown", handleGlobalKeydown);
    return () => window.removeEventListener("keydown", handleGlobalKeydown);
  });

  const handleGlobalKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.altKey || event.metaKey || event.key.length > 1)
      return;
    if (document.activeElement?.tagName.toLowerCase() !== "input") {
      campoBuscaElement?.focus();
    }
  };

  function mostrarModal(
    tipo: "sucesso" | "erro",
    titulo: string,
    mensagem: string,
  ) {
    modal = {
      tipo,
      titulo,
      mensagem,
      visible: true,
    };
  }

  function fecharModal() {
    modal.visible = false;
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

  function iniciarEdicao(tipo: "nome" | "ip" | "dns") {
    editandoNome = tipo === "nome";
    editandoIP = tipo === "ip";
    editandoDNS = tipo === "dns";

    if (tipo === "nome") tempNomeComputador = info?.nomeComputador || "";
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

  async function salvarNomeComputador() {
    if (!tempNomeComputador.trim()) {
      mostrarModal(
        "erro",
        "Erro de Validação",
        "Nome do computador não pode estar vazio!",
      );
      return;
    }
    salvandoNome = true;
    try {
      await AlterarNomeComputador(tempNomeComputador.trim());
      mostrarModal(
        "sucesso",
        "Sucesso!",
        "Nome do computador alterado! Reinicie o computador para aplicar as mudanças.",
      );
      await new Promise((resolve) => setTimeout(resolve, 1500));
      await carregarInformacoes();
      editandoNome = false;
    } catch (e) {
      mostrarModal("erro", "Erro", `Erro ao alterar nome: ${e}`);
    }
    salvandoNome = false;
  }

  async function salvarIP() {
    if (
      !tempEnderecoIP.trim() ||
      !tempMascaraRede.trim() ||
      !tempGatewayPadrao.trim()
    ) {
      mostrarModal(
        "erro",
        "Erro de Validação",
        "Todos os campos (IP, Máscara e Gateway) devem ser preenchidos!",
      );
      return;
    }
    if (!info?.interfaceAtiva) {
      mostrarModal(
        "erro",
        "Erro de Sistema",
        "Interface de rede não encontrada. Tente recarregar.",
      );
      return;
    }
    salvandoIP = true;
    try {
      await AlterarIP(
        info.interfaceAtiva,
        tempEnderecoIP,
        tempMascaraRede,
        tempGatewayPadrao,
      );
      mostrarModal(
        "sucesso",
        "Sucesso!",
        "Configurações de rede alteradas! A conexão pode ser restabelecida em breve.",
      );
      await new Promise((resolve) => setTimeout(resolve, 1500));
      await carregarInformacoes();
      editandoIP = false;
    } catch (e) {
      mostrarModal("erro", "Erro", `Erro ao alterar IP: ${e}`);
    }
    salvandoIP = false;
  }

  async function salvarDNS() {
    if (!tempDNSPrimario.trim()) {
      mostrarModal(
        "erro",
        "Erro de Validação",
        "DNS primário não pode estar vazio!",
      );
      return;
    }
    if (!info?.interfaceAtiva) {
      mostrarModal(
        "erro",
        "Erro de Sistema",
        "Interface de rede não encontrada. Tente recarregar.",
      );
      return;
    }
    salvandoDNS = true;
    try {
      await AlterarDNS(
        info.interfaceAtiva,
        tempDNSPrimario.trim(),
        tempDNSSecundario.trim(),
      );
      mostrarModal(
        "sucesso",
        "Sucesso!",
        "Servidores DNS alterados com sucesso!",
      );
      await carregarInformacoes();
      editandoDNS = false;
    } catch (e) {
      mostrarModal("erro", "Erro", `Erro ao alterar DNS: ${e}`);
    }
    salvandoDNS = false;
  }

  function cancelarEdicao() {
    editandoNome = false;
    editandoIP = false;
    editandoDNS = false;
  }

  function navegarPara(funcionalidade: string) {
    dispatch("navigate", funcionalidade);
  }
</script>

<div
  class="w-full md:max-w-7xl h-full p-5 box-border bg-primary-purple bg-opacity-10 backdrop-blur-md border border-gray-700 rounded-xl animate-fadeIn flex flex-col md:flex-row gap-5 overflow-hidden"
>
  <div
    class="flex-none w-full md:w-96 p-5 rounded-lg overflow-y-auto bg-dark-blue-light bg-opacity-25"
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
            <div
              class="bg-dark-blue-bg p-3 rounded-md border-l-4 border-primary-purple"
            >
              <span class="block text-xs text-center opacity-70 uppercase mb-1"
                >Computador</span
              >
              <div class="flex items-start gap-2 min-h-6 w-full relative">
                {#if editandoNome}
                  <div class="flex items-center gap-2 w-full">
                    <input
                      type="text"
                      bind:value={tempNomeComputador}
                      class="flex-grow p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border"
                      disabled={salvandoNome}
                    />
                    <div class="flex gap-1 flex-shrink-0">
                      <button
                        on:click={salvarNomeComputador}
                        disabled={salvandoNome}
                        class="p-1 px-1.5 border-none rounded-md cursor-pointer text-sm min-w-6 h-6 flex items-center justify-center flex-shrink-0 bg-dark-blue-light text-white hover:bg-green-600 disabled:opacity-60 disabled:cursor-not-allowed"
                        >{salvandoNome ? "..." : "✓"}</button
                      >
                      <button
                        on:click={cancelarEdicao}
                        disabled={salvandoNome}
                        class="p-1 px-1.5 border-none rounded-md cursor-pointer text-sm min-w-6 h-6 flex items-center justify-center flex-shrink-0 bg-red-600 text-white hover:bg-red-700 disabled:opacity-60 disabled:cursor-not-allowed"
                        >✕</button
                      >
                    </div>
                  </div>
                {:else}
                  <div class="flex items-start justify-center w-full relative">
                    <span
                      class="text-sm font-semibold break-all leading-snug text-center w-full"
                      >{info.nomeComputador}</span
                    >
                    <button
                      on:click={() => iniciarEdicao("nome")}
                      class="absolute right-1 top-1/2 -translate-y-1/2 p-0.5 bg-transparent text-accent-orange opacity-70 transition-all duration-200 hover:opacity-100 hover:bg-yellow-800 hover:bg-opacity-20 rounded-md"
                      aria-label="Editar nome do computador"
                    >
                      <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                      >
                        <path
                          d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                        />
                        <path d="m18.5 2.5 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                      </svg>
                    </button>
                  </div>
                {/if}
              </div>
            </div>

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
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border"
                      disabled={salvandoIP}
                      placeholder="Endereço IP"
                    />
                    <input
                      type="text"
                      bind:value={tempMascaraRede}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border"
                      disabled={salvandoIP}
                      placeholder="Máscara de Sub-rede"
                    />
                    <input
                      type="text"
                      bind:value={tempGatewayPadrao}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border"
                      disabled={salvandoIP}
                      placeholder="Gateway Padrão"
                    />
                    <div class="flex gap-1 justify-end mt-1">
                      <button
                        on:click={salvarIP}
                        disabled={salvandoIP}
                        class="p-1 px-1.5 border-none rounded-md cursor-pointer text-sm min-w-6 h-6 flex items-center justify-center flex-shrink-0 bg-dark-blue-light hover:bg-green-600 text-white disabled:opacity-60 disabled:cursor-not-allowed"
                        >{salvandoIP ? "..." : "✓"}</button
                      >
                      <button
                        on:click={cancelarEdicao}
                        disabled={salvandoIP}
                        class="p-1 px-1.5 border-none rounded-md cursor-pointer text-sm min-w-6 h-6 flex items-center justify-center flex-shrink-0 bg-red-600 text-white hover:bg-red-700 disabled:opacity-60 disabled:cursor-not-allowed"
                        >✕</button
                      >
                    </div>
                  </div>
                {:else}
                  <div class="flex items-start justify-center w-full relative">
                    <span
                      class="text-sm font-semibold break-all leading-snug text-center w-full"
                    >
                      IP: {info.enderecoIP}<br />Máscara: {info.mascaraRede}<br
                      />Gateway: {info.gatewayPadrao}
                    </span>
                    <button
                      on:click={() => iniciarEdicao("ip")}
                      class="absolute right-1 top-1/2 -translate-y-1/2 p-0.5 bg-transparent text-accent-orange opacity-70 transition-all duration-200 hover:opacity-100 hover:bg-yellow-800 hover:bg-opacity-20 rounded-md"
                      aria-label="Editar configurações de rede"
                    >
                      <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                      >
                        <path
                          d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                        />
                        <path d="m18.5 2.5 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                      </svg>
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
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border"
                      disabled={salvandoDNS}
                      placeholder="DNS Primário (ex: 8.8.8.8)"
                    />
                    <input
                      type="text"
                      bind:value={tempDNSSecundario}
                      class="p-1.5 border border-primary-purple rounded bg-dark-blue-light text-text-light text-sm box-border"
                      disabled={salvandoDNS}
                      placeholder="DNS Secundário (opcional)"
                    />
                    <div class="flex gap-1 justify-end mt-1">
                      <button
                        on:click={salvarDNS}
                        disabled={salvandoDNS}
                        class="p-1 px-1.5 border-none rounded-md cursor-pointer text-sm min-w-6 h-6 flex items-center justify-center flex-shrink-0 text-white bg-dark-blue-light hover:bg-green-600 disabled:opacity-60 disabled:cursor-not-allowed"
                        >{salvandoDNS ? "..." : "✓"}</button
                      >
                      <button
                        on:click={cancelarEdicao}
                        disabled={salvandoDNS}
                        class="p-1 px-1.5 border-none rounded-md cursor-pointer text-sm min-w-6 h-6 flex items-center justify-center flex-shrink-0 bg-red-600 text-white hover:bg-red-700 disabled:opacity-60 disabled:cursor-not-allowed"
                        >✕</button
                      >
                    </div>
                  </div>
                {:else}
                  <div class="flex items-start justify-center w-full relative">
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
                      class="absolute right-1 top-1/2 -translate-y-1/2 p-0.5 bg-transparent text-accent-orange opacity-70 transition-all duration-200 hover:opacity-100 hover:bg-yellow-800 hover:bg-opacity-20 rounded-md"
                      aria-label="Editar servidores DNS"
                    >
                      <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                      >
                        <path
                          d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                        />
                        <path d="m18.5 2.5 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                      </svg>
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

  <div class="flex-grow flex flex-col min-w-0">
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
      class="flex-grow overflow-y-auto pr-2 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 pb-3 content-start"
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

    {#if totalPaginas > 1}
      <div class="flex justify-center items-center gap-2 mt-4 flex-shrink-0">
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
  <div
    class="fixed inset-0 bg-black bg-opacity-60 backdrop-blur-sm flex items-center justify-center z-[1000] animate-modalFadeIn"
    on:click={fecharModal}
    on:keydown={(e) => {
      if (e.key === "Escape") fecharModal();
    }}
    role="button"
    tabindex="0"
    aria-label="Fechar modal"
  >
    <div
      class="bg-dark-blue-light bg-opacity-95 border border-gray-700 rounded-xl min-w-[400px] max-w-lg max-h-[80vh] overflow-hidden animate-modalSlideIn shadow-2xl"
      on:click|stopPropagation
      on:keydown={(e) => {
        if (e.key === "Escape") fecharModal();
      }}
      role="dialog"
      tabindex="0"
      aria-modal="true"
      aria-labelledby="modal-title"
      aria-describedby="modal-message"
    >
      <div
        class="flex items-center p-5 gap-3 border-b border-gray-800
        {modal.tipo === 'sucesso'
          ? 'bg-green-700/20 border-green-700/30'
          : 'bg-red-700/20 border-red-700/30'}"
      >
        <div
          class="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center
          {modal.tipo === 'sucesso'
            ? 'bg-green-700/20 text-green-500'
            : 'bg-red-700/20 text-red-500'}"
        >
          {#if modal.tipo === "sucesso"}
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="3"
            >
              <polyline points="20,6 9,17 4,12" />
            </svg>
          {:else}
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="3"
            >
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
          {/if}
        </div>
        <h3
          id="modal-title"
          class="flex-grow m-0 text-lg font-semibold text-text-light"
        >
          {modal.titulo}
        </h3>
        <button
          class="bg-transparent border-none text-gray-400 cursor-pointer p-1 rounded-md transition-all duration-200 flex-shrink-0 hover:bg-white hover:bg-opacity-10 hover:text-white"
          on:click={fecharModal}
          aria-label="Fechar"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
      <div id="modal-message" class="p-5">
        <p class="m-0 text-gray-200 leading-normal text-base">
          {modal.mensagem}
        </p>
      </div>
      <div class="px-5 pb-5 flex justify-end">
        <button
          class="bg-primary-purple text-white border-none py-2.5 px-5 rounded-md text-base font-medium cursor-pointer transition-all duration-200 hover:bg-primary-purple-dark hover:-translate-y-px"
          on:click={fecharModal}>Entendi</button
        >
      </div>
    </div>
  </div>
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

  .animate-modalFadeIn {
    animation: modalFadeIn 0.2s ease-out;
  }
  .animate-modalSlideIn {
    animation: modalSlideIn 0.3s ease-out;
  }
</style>

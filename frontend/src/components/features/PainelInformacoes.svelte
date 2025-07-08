<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    ObterInformacoesSistema,
    AlterarIP,
    AlterarNomeComputador,
    AlterarDNS,
  } from "../../../wailsjs/go/main/App";

  export let modulos = [];
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

  let termoBusca = "";
  $: funcionalidadesFiltradas = modulos
    .flatMap((m) => m.funcionalidades)
    .filter((f) => f !== "Painel de Informações")
    .sort((a, b) => a.localeCompare(b))
    .filter((f) => f.toLowerCase().includes(termoBusca.toLowerCase()));

  onMount(async () => {
    await carregarInformacoes();
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
      tempDNSPrimario = info?.dnsPrimario !== "N/A" ? info.dnsPrimario : "";
      tempDNSSecundario =
        info?.dnsSecundario !== "N/A" ? info.dnsSecundario : "";
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

<div class="dashboard-container">
  <div class="info-coluna">
    <h2>Painel de Informações</h2>
    {#if erro}
      <p class="erro-painel">{erro}</p>
    {:else if !info}
      <p>Carregando...</p>
    {:else}
      <div class="info-grid">
        <div class="info-item">
          <span class="label">Computador</span>
          <div class="value-container">
            {#if editandoNome}
              <div class="edit-container">
                <input
                  type="text"
                  bind:value={tempNomeComputador}
                  class="edit-input"
                  disabled={salvandoNome}
                />
                <div class="edit-buttons">
                  <button
                    on:click={salvarNomeComputador}
                    disabled={salvandoNome}
                    class="btn-save">{salvandoNome ? "..." : "✓"}</button
                  >
                  <button
                    on:click={cancelarEdicao}
                    disabled={salvandoNome}
                    class="btn-cancel">✕</button
                  >
                </div>
              </div>
            {:else}
              <div class="display-container">
                <span class="value">{info.nomeComputador}</span>
                <button on:click={() => iniciarEdicao("nome")} class="btn-edit">
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

        <div class="info-item">
          <span class="label">RAM</span><span class="value"
            >{info.memoriaTotalGB}</span
          >
        </div>
        <div class="info-item full-width">
          <span class="label">Windows</span><span class="value"
            >{info.edicaoWindows} ({info.versaoWindows})</span
          >
        </div>
        <div class="info-item full-width">
          <span class="label">Processador</span><span class="value"
            >{info.processador}</span
          >
        </div>
        <div class="info-item">
          <span class="label">MAC</span><span class="value"
            >{info.enderecoMAC}</span
          >
        </div>

        <div class="info-item full-width">
          <span class="label">Rede IPv4</span>
          <div class="value-container">
            {#if editandoIP}
              <div class="multi-edit-container">
                <input
                  type="text"
                  bind:value={tempEnderecoIP}
                  class="edit-input"
                  disabled={salvandoIP}
                  placeholder="Endereço IP"
                />
                <input
                  type="text"
                  bind:value={tempMascaraRede}
                  class="edit-input"
                  disabled={salvandoIP}
                  placeholder="Máscara de Sub-rede"
                />
                <input
                  type="text"
                  bind:value={tempGatewayPadrao}
                  class="edit-input"
                  disabled={salvandoIP}
                  placeholder="Gateway Padrão"
                />
                <div class="edit-buttons">
                  <button
                    on:click={salvarIP}
                    disabled={salvandoIP}
                    class="btn-save">{salvandoIP ? "..." : "✓"}</button
                  >
                  <button
                    on:click={cancelarEdicao}
                    disabled={salvandoIP}
                    class="btn-cancel">✕</button
                  >
                </div>
              </div>
            {:else}
              <div class="display-container">
                <span class="value"
                  >IP: {info.enderecoIP}<br />Máscara: {info.mascaraRede}<br
                  />Gateway: {info.gatewayPadrao}</span
                >
                <button on:click={() => iniciarEdicao("ip")} class="btn-edit">
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

        <div class="info-item full-width">
          <span class="label">DNS</span>
          <div class="value-container">
            {#if editandoDNS}
              <div class="multi-edit-container">
                <input
                  type="text"
                  bind:value={tempDNSPrimario}
                  class="edit-input"
                  disabled={salvandoDNS}
                  placeholder="DNS Primário (ex: 8.8.8.8)"
                />
                <input
                  type="text"
                  bind:value={tempDNSSecundario}
                  class="edit-input"
                  disabled={salvandoDNS}
                  placeholder="DNS Secundário (opcional)"
                />
                <div class="edit-buttons">
                  <button
                    on:click={salvarDNS}
                    disabled={salvandoDNS}
                    class="btn-save">{salvandoDNS ? "..." : "✓"}</button
                  >
                  <button
                    on:click={cancelarEdicao}
                    disabled={salvandoDNS}
                    class="btn-cancel">✕</button
                  >
                </div>
              </div>
            {:else}
              <div class="display-container">
                <span class="value">
                  {#if info.dnsPrimario && info.dnsPrimario !== "N/A"}
                    Primário: {info.dnsPrimario}
                    {#if info.dnsSecundario && info.dnsSecundario !== "N/A"}
                      <br />Secundário: {info.dnsSecundario}
                    {/if}
                  {:else}
                    Não configurado ou Automático
                  {/if}
                </span>
                <button on:click={() => iniciarEdicao("dns")} class="btn-edit">
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
    {/if}
  </div>

  <div class="modulos-coluna">
    <h3>Todas as Ferramentas ({funcionalidadesFiltradas.length})</h3>
    <input
      type="search"
      class="campo-busca"
      placeholder="Pesquisar ferramenta..."
      bind:value={termoBusca}
      bind:this={campoBuscaElement}
    />
    <div class="botoes-grid">
      {#each funcionalidadesFiltradas as funcionalidade (funcionalidade)}
        <button class="btn-funcao" on:click={() => navegarPara(funcionalidade)}
          >{funcionalidade}</button
        >
      {/each}
      {#if funcionalidadesFiltradas.length === 0}
        <p class="nenhum-resultado">Nenhuma ferramenta encontrada.</p>
      {/if}
    </div>
  </div>
</div>

<!-- Modal Personalizado -->
{#if modal.visible}
  <div class="modal-overlay" on:click={fecharModal}>
    <div class="modal-container" on:click|stopPropagation>
      <div class="modal-header {modal.tipo}">
        <div class="modal-icon">
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
        <h3 class="modal-titulo">{modal.titulo}</h3>
        <button class="modal-close" on:click={fecharModal}>
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
      <div class="modal-body">
        <p class="modal-mensagem">{modal.mensagem}</p>
      </div>
      <div class="modal-footer">
        <button class="modal-btn" on:click={fecharModal}>Entendi</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dashboard-container {
    width: clamp(800px, 98vw, 1600px);
    height: clamp(550px, 98vh, 800px);
    padding: 20px;
    box-sizing: border-box;
    background-color: rgba(28, 32, 114, 0.374);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(240, 240, 240, 0.3);
    border-radius: 12px;
    animation: fadeIn 0.5s ease-out;
    display: flex;
    gap: 25px;
  }
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  h2,
  h3 {
    margin-top: 0;
    color: var(--accent-orange);
  }
  .info-coluna {
    flex-basis: 400px;
    flex-shrink: 0;
    background-color: rgba(17, 22, 114, 0.285);
    padding: 20px;
    border-radius: 8px;
    overflow-y: auto;
  }
  .info-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 12px;
  }
  .info-item {
    background-color: var(--bg-light);
    padding: 8px 12px;
    border-radius: 6px;
    border-left: 4px solid var(--accent-blue);
  }
  .info-item .label {
    display: block;
    font-size: 0.7rem;
    opacity: 0.7;
    text-transform: uppercase;
    margin-bottom: 4px;
  }
  .value-container {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    min-height: 24px;
    width: 100%;
  }
  .info-item .value {
    font-size: 0.9rem;
    font-weight: 600;
    word-break: break-all;
    line-height: 1.4;
    text-align: center;
    width: 100%;
  }
  .display-container {
    display: flex;
    align-items: flex-start;
    justify-content: center;
    width: 100%;
    position: relative;
  }
  .edit-container {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
  .edit-input {
    flex-grow: 1;
    padding: 6px 8px;
    border: 1px solid var(--accent-blue);
    border-radius: 4px;
    background-color: rgba(0, 0, 0, 0.3);
    color: var(--text-light);
    font-size: 0.85rem;
    box-sizing: border-box;
  }
  .multi-edit-container {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }
  .multi-edit-container .edit-buttons {
    align-self: flex-end;
  }
  .edit-buttons {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
  .btn-edit,
  .btn-save,
  .btn-cancel {
    padding: 4px 6px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
    min-width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .btn-edit {
    background-color: transparent;
    color: var(--accent-orange);
    opacity: 0.7;
    transition: all 0.2s;
    padding: 2px;
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
  }
  .btn-edit:hover {
    opacity: 1;
    background-color: rgba(255, 165, 0, 0.1);
    border-radius: 4px;
  }
  .btn-save {
    background-color: var(--accent-blue);
    color: white;
  }
  .btn-cancel {
    background-color: #dc3545;
    color: white;
  }
  .btn-save:hover {
    background-color: #0056b3;
  }
  .btn-cancel:hover {
    background-color: #c82333;
  }
  .btn-save:disabled,
  .btn-cancel:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .modulos-coluna {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .campo-busca {
    flex-shrink: 0;
    width: 100%;
    box-sizing: border-box;
    padding: 10px;
    margin-bottom: 15px;
    background-color: rgba(12, 16, 89, 0.7);
    border: 1px solid var(--accent-blue);
    color: var(--text-light);
    border-radius: 8px;
    font-size: 1rem;
  }
  .botoes-grid {
    flex-grow: 1;
    overflow-y: auto;
    padding-right: 10px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 12px;
    align-content: flex-start;
  }
  .btn-funcao {
    padding: 15px;
    border: 1px solid var(--bg-light);
    background-color: var(--bg-light);
    color: var(--text-light);
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: bold;
    text-align: left;
    transition: all 0.2s ease;
  }
  .btn-funcao:hover {
    background-color: var(--accent-blue);
    transform: translateY(-2px);
    border-color: var(--accent-orange);
  }
  .erro-painel,
  .nenhum-resultado {
    opacity: 0.8;
    text-align: center;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: modalFadeIn 0.2s ease-out;
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

  .modal-container {
    background-color: rgba(25, 28, 89, 0.95);
    border: 1px solid rgba(240, 240, 240, 0.2);
    border-radius: 12px;
    min-width: 400px;
    max-width: 500px;
    max-height: 80vh;
    overflow: hidden;
    animation: modalSlideIn 0.3s ease-out;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  }

  .modal-header {
    display: flex;
    align-items: center;
    padding: 20px;
    gap: 12px;
    border-bottom: 1px solid rgba(240, 240, 240, 0.1);
  }

  .modal-header.sucesso {
    background: linear-gradient(
      135deg,
      rgba(34, 197, 94, 0.2),
      rgba(34, 197, 94, 0.1)
    );
    border-bottom-color: rgba(34, 197, 94, 0.3);
  }

  .modal-header.erro {
    background: linear-gradient(
      135deg,
      rgba(239, 68, 68, 0.2),
      rgba(239, 68, 68, 0.1)
    );
    border-bottom-color: rgba(239, 68, 68, 0.3);
  }

  .modal-icon {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-header.sucesso .modal-icon {
    background-color: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .modal-header.erro .modal-icon {
    background-color: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .modal-titulo {
    flex-grow: 1;
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-light);
  }

  .modal-close {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.6);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    transition: all 0.2s;
    flex-shrink: 0;
  }

  .modal-close:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.9);
  }

  .modal-body {
    padding: 20px;
  }

  .modal-mensagem {
    margin: 0;
    color: rgba(255, 255, 255, 0.9);
    line-height: 1.5;
    font-size: 0.95rem;
  }

  .modal-footer {
    padding: 0 20px 20px;
    display: flex;
    justify-content: flex-end;
  }

  .modal-btn {
    background-color: var(--accent-blue);
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .modal-btn:hover {
    background-color: #0056b3;
    transform: translateY(-1px);
  }

  @media (max-width: 900px) {
    .dashboard-container {
      flex-direction: column;
      width: 98vw;
      height: 98vh;
      overflow-y: auto;
      padding: 15px;
    }
    .info-coluna {
      flex-basis: auto;
    }
    .modulos-coluna {
      min-height: 300px;
    }

    .modal-container {
      min-width: 90vw;
      max-width: 90vw;
      margin: 20px;
    }
  }
</style>

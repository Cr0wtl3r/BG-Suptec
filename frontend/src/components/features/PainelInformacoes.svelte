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
    if (event.ctrlKey || event.altKey || event.metaKey || event.key.length > 1) return;
    if (document.activeElement?.tagName.toLowerCase() !== "input") {
      campoBuscaElement?.focus();
    }
  };

  async function carregarInformacoes() {
    try {
      info = await ObterInformacoesSistema();
      erro = "";
    } catch (e) {
      erro = `Erro ao carregar informações: ${e}`;
      info = null;
    }
  }

  function iniciarEdicao(tipo: 'nome' | 'ip' | 'dns') {
    editandoNome = tipo === 'nome';
    editandoIP = tipo === 'ip';
    editandoDNS = tipo === 'dns';

    if (tipo === 'nome') tempNomeComputador = info?.nomeComputador || "";
    if (tipo === 'ip') {
      tempEnderecoIP = info?.enderecoIP || "";
      tempMascaraRede = info?.mascaraRede || "255.255.255.0";
      tempGatewayPadrao = info?.gatewayPadrao || "";
    }
    if (tipo === 'dns') {
      tempDNSPrimario = info?.dnsPrimario !== "N/A" ? info.dnsPrimario : "";
      tempDNSSecundario = info?.dnsSecundario !== "N/A" ? info.dnsSecundario : "";
    }
  }

  async function salvarNomeComputador() {
    if (!tempNomeComputador.trim()) {
      alert("Nome do computador não pode estar vazio!");
      return;
    }
    salvandoNome = true;
    try {
      await AlterarNomeComputador(tempNomeComputador.trim());
      alert("Nome do computador alterado! Reinicie o computador para aplicar as mudanças.");
      await carregarInformacoes();
      editandoNome = false;
    } catch (e) {
      alert(`Erro ao alterar nome: ${e}`);
    }
    salvandoNome = false;
  }

  async function salvarIP() {
    if (!tempEnderecoIP.trim() || !tempMascaraRede.trim() || !tempGatewayPadrao.trim()) {
      alert("Todos os campos (IP, Máscara e Gateway) devem ser preenchidos!");
      return;
    }
    if (!info?.interfaceAtiva) {
      alert("Erro: Interface de rede não encontrada. Tente recarregar.");
      return;
    }
    salvandoIP = true;
    try {
      await AlterarIP(info.interfaceAtiva, tempEnderecoIP, tempMascaraRede, tempGatewayPadrao);
      alert("Configurações de rede alteradas! A conexão pode ser restabelecida em breve.");
      await new Promise(resolve => setTimeout(resolve, 4000)); // Espera a rede estabilizar
      await carregarInformacoes();
      editandoIP = false;
    } catch (e) {
      alert(`Erro ao alterar IP: ${e}`);
    }
    salvandoIP = false;
  }

  async function salvarDNS() {
    if (!tempDNSPrimario.trim()) {
      alert("DNS primário não pode estar vazio!");
      return;
    }
    if (!info?.interfaceAtiva) {
      alert("Erro: Interface de rede não encontrada. Tente recarregar.");
      return;
    }
    salvandoDNS = true;
    try {
      await AlterarDNS(info.interfaceAtiva, tempDNSPrimario.trim(), tempDNSSecundario.trim());
      alert("Servidores DNS alterados com sucesso!");
      await carregarInformacoes();
      editandoDNS = false;
    } catch (e) {
      alert(`Erro ao alterar DNS: ${e}`);
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
              <input type="text" bind:value={tempNomeComputador} class="edit-input" disabled={salvandoNome} />
              <div class="edit-buttons">
                <button on:click={salvarNomeComputador} disabled={salvandoNome} class="btn-save">{salvandoNome ? "..." : "✓"}</button>
                <button on:click={cancelarEdicao} disabled={salvandoNome} class="btn-cancel">✕</button>
              </div>
            {:else}
              <span class="value">{info.nomeComputador}</span>
              <button on:click={() => iniciarEdicao('nome')} class="btn-edit">✏️</button>
            {/if}
          </div>
        </div>

        <div class="info-item"><span class="label">RAM</span><span class="value">{info.memoriaTotalGB}</span></div>
        <div class="info-item full-width"><span class="label">Windows</span><span class="value">{info.edicaoWindows} ({info.versaoWindows})</span></div>
        <div class="info-item full-width"><span class="label">Processador</span><span class="value">{info.processador}</span></div>
        <div class="info-item"><span class="label">MAC</span><span class="value">{info.enderecoMAC}</span></div>

        <div class="info-item full-width">
          <span class="label">Rede IPv4</span>
          <div class="value-container">
            {#if editandoIP}
              <div class="multi-edit-container">
                <input type="text" bind:value={tempEnderecoIP} class="edit-input" disabled={salvandoIP} placeholder="Endereço IP" />
                <input type="text" bind:value={tempMascaraRede} class="edit-input" disabled={salvandoIP} placeholder="Máscara de Sub-rede" />
                <input type="text" bind:value={tempGatewayPadrao} class="edit-input" disabled={salvandoIP} placeholder="Gateway Padrão" />
                <div class="edit-buttons">
                  <button on:click={salvarIP} disabled={salvandoIP} class="btn-save">{salvandoIP ? "..." : "✓"}</button>
                  <button on:click={cancelarEdicao} disabled={salvandoIP} class="btn-cancel">✕</button>
                </div>
              </div>
            {:else}
              <div class="display-container">
                <span class="value">IP: {info.enderecoIP}<br />Máscara: {info.mascaraRede}<br />Gateway: {info.gatewayPadrao}</span>
                <button on:click={() => iniciarEdicao('ip')} class="btn-edit">✏️</button>
              </div>
            {/if}
          </div>
        </div>

        <div class="info-item full-width">
          <span class="label">DNS</span>
          <div class="value-container">
            {#if editandoDNS}
              <div class="multi-edit-container">
                <input type="text" bind:value={tempDNSPrimario} class="edit-input" disabled={salvandoDNS} placeholder="DNS Primário (ex: 8.8.8.8)" />
                <input type="text" bind:value={tempDNSSecundario} class="edit-input" disabled={salvandoDNS} placeholder="DNS Secundário (opcional)" />
                <div class="edit-buttons">
                  <button on:click={salvarDNS} disabled={salvandoDNS} class="btn-save">{salvandoDNS ? "..." : "✓"}</button>
                  <button on:click={cancelarEdicao} disabled={salvandoDNS} class="btn-cancel">✕</button>
                </div>
              </div>
            {:else}
              <div class="display-container">
                <span class="value">
                  {#if info.dnsPrimario && info.dnsPrimario !== 'N/A'}
                    Primário: {info.dnsPrimario}
                    {#if info.dnsSecundario && info.dnsSecundario !== 'N/A'}
                      <br />Secundário: {info.dnsSecundario}
                    {/if}
                  {:else}
                    Não configurado ou Automático
                  {/if}
                </span>
                <button on:click={() => iniciarEdicao('dns')} class="btn-edit">✏️</button>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>

  <div class="modulos-coluna">
    <h3>Todas as Ferramentas ({funcionalidadesFiltradas.length})</h3>
    <input type="search" class="campo-busca" placeholder="Pesquisar ferramenta..." bind:value={termoBusca} bind:this={campoBuscaElement} />
    <div class="botoes-grid">
      {#each funcionalidadesFiltradas as funcionalidade (funcionalidade)}
        <button class="btn-funcao" on:click={() => navegarPara(funcionalidade)}>{funcionalidade}</button>
      {/each}
      {#if funcionalidadesFiltradas.length === 0}
        <p class="nenhum-resultado">Nenhuma ferramenta encontrada.</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .dashboard-container {
    width: clamp(800px, 98vw, 1600px);
    height: clamp(550px, 98vh, 800px);
    padding: 20px;
    box-sizing: border-box;
    background-color: rgba(25, 28, 89, 0.4);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(240, 240, 240, 0.3);
    border-radius: 12px;
    animation: fadeIn 0.5s ease-out;
    display: flex;
    gap: 25px;
  }
  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }

  h2, h3 { margin-top: 0; color: var(--accent-orange); }
  .info-coluna {
    flex-basis: 400px;
    flex-shrink: 0;
    background-color: rgba(12, 16, 89, 0.5);
    padding: 20px;
    border-radius: 8px;
    overflow-y: auto;
  }
  .info-grid { display: grid; grid-template-columns: 1fr; gap: 12px; }
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
  .value-container { display: flex; align-items: center; gap: 8px; min-height: 24px; }
  .info-item .value { font-size: 0.9rem; font-weight: 600; word-break: break-all; flex-grow: 1; line-height: 1.4; }
  .display-container { display: flex; align-items: flex-start; justify-content: space-between; width: 100%; }
  .edit-input {
    flex-grow: 1;
    padding: 6px 8px;
    border: 1px solid var(--accent-blue);
    border-radius: 4px;
    background-color: rgba(0, 0, 0, 0.3);
    color: var(--text-light);
    font-size: 0.85rem;
    width: 100%;
    box-sizing: border-box;
  }
  .multi-edit-container { display: flex; flex-direction: column; gap: 6px; width: 100%; }
  .multi-edit-container .edit-buttons { align-self: flex-end; }
  .edit-buttons { display: flex; gap: 4px; }
  .btn-edit, .btn-save, .btn-cancel {
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
  }
  .btn-edit { background-color: transparent; color: var(--accent-orange); opacity: 0.7; transition: opacity 0.2s; }
  .btn-edit:hover { opacity: 1; }
  .btn-save { background-color: var(--accent-blue); color: white; }
  .btn-cancel { background-color: #dc3545; color: white; }
  .btn-save:hover { background-color: #0056b3; }
  .btn-cancel:hover { background-color: #c82333; }
  .btn-save:disabled, .btn-cancel:disabled { opacity: 0.6; cursor: not-allowed; }
  .modulos-coluna { flex-grow: 1; display: flex; flex-direction: column; min-width: 0; }
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
  .btn-funcao:hover { background-color: var(--accent-blue); transform: translateY(-2px); border-color: var(--accent-orange); }
  .erro-painel, .nenhum-resultado { opacity: 0.8; text-align: center; }

  @media (max-width: 900px) {
    .dashboard-container { flex-direction: column; width: 98vw; height: 98vh; overflow-y: auto; padding: 15px; }
    .info-coluna { flex-basis: auto; }
    .modulos-coluna { min-height: 300px; }
  }
</style>
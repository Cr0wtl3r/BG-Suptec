<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { ObterInformacoesSistema } from "../../../wailsjs/go/main/App";

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
  };
  let info: InfoSistema | null = null;
  let erro = "";

  let termoBusca = "";
  let todasFuncionalidades = [];
  let campoBuscaElement: HTMLInputElement;

  $: if (modulos.length > 0 && todasFuncionalidades.length === 0) {
    todasFuncionalidades = modulos
      .flatMap((m) => m.funcionalidades)
      .filter((f) => f !== "Painel de Informações")
      .sort((a, b) => a.localeCompare(b));
  }

  $: funcionalidadesFiltradas = todasFuncionalidades.filter((f) =>
    f.toLowerCase().includes(termoBusca.toLowerCase()),
  );

  onMount(async () => {
    try {
      info = await ObterInformacoesSistema();
    } catch (e) {
      erro = `Erro ao carregar informações: ${e}`;
    }

    const handleGlobalKeydown = (event: KeyboardEvent) => {
      if (
        event.ctrlKey ||
        event.altKey ||
        event.metaKey ||
        event.key.length > 1
      )
        return;
      if (document.activeElement?.tagName.toLowerCase() !== "input") {
        campoBuscaElement?.focus();
      }
    };
    window.addEventListener("keydown", handleGlobalKeydown);

    return () => {
      window.removeEventListener("keydown", handleGlobalKeydown);
    };
  });

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
          <span class="label">Computador</span><span class="value"
            >{info.nomeComputador}</span
          >
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
        <div class="info-item">
          <span class="label">IP</span><span class="value"
            >{info.enderecoIP}</span
          >
        </div>
      </div>
    {/if}
  </div>

  <div class="modulos-coluna">
    <h3>Todas as Ferramentas ({funcionalidadesFiltradas.length})</h3>

    <input
      type="search"
      class="campo-busca"
      placeholder="Pesquisar ferramenta... (ex: dns, office)"
      bind:value={termoBusca}
      bind:this={campoBuscaElement}
    />

    <div class="botoes-grid">
      {#each funcionalidadesFiltradas as funcionalidade (funcionalidade)}
        <button class="btn-funcao" on:click={() => navegarPara(funcionalidade)}>
          {funcionalidade}
        </button>
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
    flex-basis: 350px;
    flex-shrink: 0;
    background-color: rgba(12, 16, 89, 0.5);
    padding: 20px;
    border-radius: 8px;
    overflow-y: auto;
  }
  .info-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 10px;
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
  }
  .info-item .value {
    font-size: 0.9rem;
    font-weight: 600;
    word-break: break-all;
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
  .nenhum-resultado {
    opacity: 0.7;
    text-align: center;
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
  }
</style>

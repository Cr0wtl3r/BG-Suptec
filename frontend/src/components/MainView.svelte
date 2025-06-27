<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import AtivacaoWindows from "./features/AtivacaoWindows.svelte";
  import AtivacaoOffice from "./features/AtivacaoOffice.svelte";
  import PainelInformacoes from "./features/PainelInformacoes.svelte";
  import FeatureRunner from "./features/FeatureRunner.svelte";
  import BotaoVoltar from "./shared/BotaoVoltar.svelte";

  export let visao;
  export let modulos;
  const dispatch = createEventDispatcher();

  // Mapeamento das funcionalidades simples para seus comandos e descrições
  const comandosSimples = {
    "Limpa cache DNS": {
      cmd: "ipconfig",
      args: ["/flushdns"],
      desc: 'Executa o comando "ipconfig /flushdns" para limpar o cache de resolução de DNS local.',
    },
    "Desativa a Hibernação do Windows": {
      cmd: "powercfg",
      args: ["-h", "off"],
      desc: "Desativa o recurso de hibernação e remove o arquivo hiberfil.sys, liberando espaço em disco.",
    },
    // Adicione outras funções simples aqui no futuro
  };
</script>

{#if visao === "Painel de Informações"}
  <PainelInformacoes
    {modulos}
    on:navigate={(e) => dispatch("navigate", e.detail)}
  />
{:else if visao === "Windows - Ativação 180 dias"}
  <AtivacaoWindows
    on:voltar={() => dispatch("navigate", "Painel de Informações")}
  />
{:else if visao === "Office - Ativação 180 dias"}
  <AtivacaoOffice
    on:voltar={() => dispatch("navigate", "Painel de Informações")}
  />
{:else if comandosSimples[visao]}
  <FeatureRunner
    titulo={visao}
    descricao={comandosSimples[visao].desc}
    comando={comandosSimples[visao].cmd}
    args={comandosSimples[visao].args}
    on:voltar={() => dispatch("navigate", "Painel de Informações")}
  />
{:else}
  <div class="main-app">
    <h1>{visao}</h1>
    <p>Funcionalidade em desenvolvimento...</p>
    <BotaoVoltar
      on:click={() => dispatch("navigate", "Painel de Informações")}
    />
  </div>
{/if}

<style>
  .main-app {
    position: relative;
    z-index: 2;
    color: var(--text-light);
    font-size: 1.5rem;
    text-align: center;
    background-color: rgba(25, 28, 89, 0.2);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(240, 240, 240, 0.301);
    padding: 40px;
    border-radius: 12px;
  }
</style>

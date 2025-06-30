<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import AtivacaoWindows from "./features/AtivacaoWindows.svelte";
  import AtivacaoOffice from "./features/AtivacaoOffice.svelte";
  import LimpaCacheDNS from "./features/LimpaCacheDNS.svelte";
  import DesativaHibernacao from "./features/DesativaHibernacao.svelte";
  import LimpaSpoolImpressao from "./features/LimpaSpoolImpressao.svelte";
  import FeatureRunner from "./features/FeatureRunner.svelte";

  export let visao;

  const dispatch = createEventDispatcher();

  function handleVoltar() {
    dispatch("navigate", "Painel de Informações");
  }
</script>

{#if visao === "Painel de Informações"}
  <p>Erro: Visão do Painel deveria ser tratada pelo App.svelte</p>
{:else if visao === "Windows - Ativação 180 dias"}
  <AtivacaoWindows on:voltar={handleVoltar} />
{:else if visao === "Office - Ativação 180 dias"}
  <AtivacaoOffice on:voltar={handleVoltar} />
{:else if visao === "Limpa cache DNS"}
  <LimpaCacheDNS on:voltar={handleVoltar} />
{:else if visao === "Desativa a Hibernação do Windows"}
  <DesativaHibernacao on:voltar={handleVoltar} />
{:else if visao === "Limpa e Reinicia Spool de Impressão"}
  <LimpaSpoolImpressao on:voltar={handleVoltar} />
{:else}
  <FeatureRunner
    titulo={visao}
    descricao="Esta funcionalidade ainda está em desenvolvimento."
    textoBotao="Indisponível"
    emExecucao={true}
    on:voltar={handleVoltar}
  />
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
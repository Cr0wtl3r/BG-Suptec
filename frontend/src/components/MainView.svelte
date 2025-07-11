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

<div
  class="flex flex-col flex-grow items-center justify-center p-5 w-full h-full"
>
  {#if visao === "Painel de Informações"}
    <p>Erro: Visão do Painel deveria ser tratada pelo App.svelte</p>
  {:else if visao === "Windows - Ativação"}
    <AtivacaoWindows on:voltar={handleVoltar} />
  {:else if visao === "Office - Ativação"}
    <AtivacaoOffice on:voltar={handleVoltar} />
  {:else if visao === "Limpa cache DNS"}
    <LimpaCacheDNS on:voltar={handleVoltar} />
  {:else if visao === "Desativa a Hibernação do Windows"}
    <DesativaHibernacao on:voltar={handleVoltar} />
  {:else if visao === "Limpa e Reinicia Spool de Impressao"}
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
</div>

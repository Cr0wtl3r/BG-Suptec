<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import AtivacaoWindows from "./features/AtivacaoWindows.svelte";
  import AtivacaoOffice from "./features/AtivacaoOffice.svelte";
  import FeatureRunner from "./features/FeatureRunner.svelte";

  export let visao;

  const dispatch = createEventDispatcher();

  const featureMap = {
    "Limpa cache DNS": {
      desc: 'Executa o comando "ipconfig /flushdns" para limpar o cache de resolução de DNS local.',
      cmd: "ipconfig",
      args: ["/flushdns"],
    },
    "Desativa a Hibernação do Windows": {
      desc: "Desativa o recurso de hibernação e remove o arquivo hiberfil.sys, liberando espaço em disco.",
      cmd: "powercfg",
      args: ["-h", "off"],
    },
    "Limpa e Reinicia Spool de Impressão": {
      desc: "Para e reinicia o serviço de Spooler de Impressão, útil para resolver problemas de impressoras travadas.",
      cmd: "net",
      args: ["stop", "spooler", "&&", "net", "start", "spooler"],
    },
  };

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
{:else if featureMap[visao]}
  <FeatureRunner
    titulo={visao}
    descricao={featureMap[visao].desc}
    comando={featureMap[visao].cmd}
    args={featureMap[visao].args}
    on:voltar={handleVoltar}
  />
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
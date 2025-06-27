<script lang="ts">
  import { onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { ExecutarComandoSimples } from "../../../wailsjs/go/main/App";

  import FeatureContainer from "../shared/FeatureContainer.svelte";
  import LogPanel from "../shared/LogPanel.svelte";
  import BotaoVoltar from "../shared/BotaoVoltar.svelte";
  import { createEventDispatcher } from "svelte";

  export let titulo: string;
  export let descricao: string;
  export let comando: string;
  export let args: string[] = [];

  const dispatch = createEventDispatcher();
  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;

  async function iniciar() {
    emExecucao = true;
    logLines = [];
    try {
      await ExecutarComandoSimples(titulo, comando, ...args);
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
      emExecucao = false;
    }
  }

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    logLines = [...logLines, `[${timestamp}] ${mensagem}`];
    if (mensagem.includes("---")) {
      emExecucao = false;
    }
  }

  onMount(() => {
    const eventName = "log:runner";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureContainer {titulo}>
  <p>{descricao}</p>

  <button class="btn-executar" on:click={iniciar} disabled={emExecucao}>
    {emExecucao ? "Executando..." : `Executar "${titulo}"`}
  </button>

  <LogPanel {logLines} />

  <BotaoVoltar on:click={() => dispatch("voltar")} />
</FeatureContainer>

<style>
  p {
    margin-top: 0;
    margin-bottom: 25px;
    opacity: 0.9;
  }
</style>

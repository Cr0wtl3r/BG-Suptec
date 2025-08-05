<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AtivarProtecaoSistema } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();
  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;

  async function iniciar() {
    emExecucao = true;
    logLines = [];
    try {
      AtivarProtecaoSistema();
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
    const eventName = "log:ativar:protecao";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureRunner
  titulo="Ativar Proteção do Sistema"
  descricao="Esta função ativa a Restauração do Sistema para a unidade C: e configura o uso de disco para 5%. Isso permite reverter o computador para um estado anterior em caso de problemas. Requer privilégios de Administrador."
  textoBotao="Ativar Proteção"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
/>

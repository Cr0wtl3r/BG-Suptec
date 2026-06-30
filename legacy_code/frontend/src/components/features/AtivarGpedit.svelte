<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AtivarGpedit } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();
  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;

  async function iniciar() {
    emExecucao = true;
    logLines = [];
    try {
      AtivarGpedit();
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
    // Ouve o evento correto
    const eventName = "log:ativar:gpedit";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureRunner
  titulo="Ativar Editor de Política de Grupo (gpedit.msc)"
  descricao="Esta função instala o Editor de Política de Grupo (gpedit.msc) em edições do Windows (como a Home) que não o possuem por padrão. Requer privilégios de Administrador e pode demorar alguns minutos."
  textoBotao="Ativar o Gpedit.msc"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
/>

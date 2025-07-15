<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AjustarHoraFormatacao } from "../../../wailsjs/go/main/App";

  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;

  async function iniciar() {
    emExecucao = true;
    logLines = [];
    try {
      await AjustarHoraFormatacao();
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
    const eventName = "log:ajustar:hora:formatacao";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureRunner
  titulo="Ajustar Hora da Formatação"
  descricao="Esta função configura o serviço de horário do Windows, sincroniza com servidores NTP e ajusta a data de instalação do sistema para a data/hora atual. Requer privilégios de Administrador."
  textoBotao="Ajustar Hora"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
/>

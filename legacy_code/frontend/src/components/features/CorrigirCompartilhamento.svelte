<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import {
    CorrigirCompartilhamentoWindows,
    ReiniciarComputador,
  } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";
  import Modal from "../shared/Modal.svelte";

  const dispatch = createEventDispatcher();
  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let mostrarModalReinicio = false;

  async function iniciar() {
    emExecucao = true;
    mostrarModalReinicio = false;
    logLines = [];
    try {
      CorrigirCompartilhamentoWindows();
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
    const eventName = "log:compartilhamento";
    EventsOn(eventName, adicionarLog);
    EventsOn("compartilhamento:finalizado", () => {
      mostrarModalReinicio = true;
    });

    return () => {
      EventsOff(eventName);
      EventsOff("compartilhamento:finalizado");
    };
  });
</script>

<FeatureRunner
  titulo="Corrigir Compartilhamento de Rede"
  descricao="Aplica um conjunto completo de correções para problemas comuns de compartilhamento de pastas e impressoras em rede local. Requer privilégios de Administrador."
  textoBotao="Aplicar Correções"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
/>

{#if mostrarModalReinicio}
  <Modal
    tipo="aviso"
    titulo="Reinicialização Recomendada"
    mensagem="As correções foram aplicadas com sucesso..."
    textoConfirmar="Reiniciar Agora"
    textoCancelar="Depois"
    on:confirmar={async () => {
      try {
        logLines = [...logLines, "[INFO] Comando de reinicialização enviado."];
        await ReiniciarComputador();
        mostrarModalReinicio = false;
      } catch (err) {
        logLines = [...logLines, `[ERRO] Não foi possível reiniciar: ${err}`];
        mostrarModalReinicio = false;
      }
    }}
    on:cancelar={() => (mostrarModalReinicio = false)}
  />
{/if}

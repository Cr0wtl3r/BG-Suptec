<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime/runtime";
  import {
    CorrigirCompartilhamentoWindows,
    ReiniciarComputador,
  } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();
  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let operacaoConcluida = false; // Controla se a operação terminou

  async function iniciar() {
    emExecucao = true;
    operacaoConcluida = false;
    logLines = []; // Limpa os logs
    try {
      await CorrigirCompartilhamentoWindows(); // Chama a função principal do Go
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

  function handleReboot() {
    logLines = [...logLines, "Comando de reinicialização enviado..."];
    ReiniciarComputador().catch((err) => {
      logLines = [...logLines, `Falha ao reiniciar: ${err}`];
    });
  }

  onMount(() => {
    const eventName = "log:compartilhamento";
    EventsOn(eventName, adicionarLog);

    // Ouve o evento especial para mostrar os botões
    EventsOn("compartilhamento:finalizado", () => {
      operacaoConcluida = true;
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
>
  {#if operacaoConcluida}
    <div
      class="mt-4 p-4 border-t-2 border-accent-orange/30 text-center animate-fadeIn"
    >
      <p class="font-bold mb-3">
        Operação concluída. Uma reinicialização é recomendada.
      </p>
      <button
        class="px-5 py-2 text-base font-semibold cursor-pointer bg-accent-orange text-dark-blue-bg border-none rounded-lg transition-all duration-200 hover:brightness-110"
        on:click={handleReboot}
      >
        Reiniciar Agora
      </button>
    </div>
  {/if}
</FeatureRunner>

<style>
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  .animate-fadeIn {
    animation: fadeIn 0.5s ease-out;
  }
</style>

<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ExecutarComando } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let tempoSelecionado = "60";
  let agendamentoAtivo = false;

  async function iniciar() {
    emExecucao = true;
    agendamentoAtivo = false;
    logLines = ["Iniciando agendamento de desligamento..."];

    try {
      adicionarLog("Verificando e cancelando qualquer agendamento anterior...");
      await ExecutarComando("shutdown", ["/a"]);
      adicionarLog("Agendamento anterior cancelado (se existia).");
      await new Promise((resolve) => setTimeout(resolve, 500));

      adicionarLog(`Agendando desligamento em ${tempoSelecionado} segundos...`);
      const resultado = await ExecutarComando("shutdown", [
        "/s",
        "/t",
        tempoSelecionado,
      ]);
      adicionarLog("Resultado do comando: " + resultado);
      adicionarLog(
        `Seu computador será desligado em ${tempoSelecionado} segundos.`,
      );
      agendamentoAtivo = true;
    } catch (err) {
      adicionarLog(`ERRO: ${err}`);
      agendamentoAtivo = false;
    } finally {
      adicionarLog("--- Operação de agendamento concluída ---");
    }
  }

  async function cancelarDesligamento() {
    emExecucao = true;
    logLines = ["Tentando cancelar o desligamento agendado..."];
    try {
      adicionarLog("Executando comando: shutdown /a");
      const resultado = await ExecutarComando("shutdown", ["/a"]);
      adicionarLog("Resultado do comando: " + resultado);
      adicionarLog("Desligamento agendado cancelado com sucesso!");
      agendamentoAtivo = false;
    } catch (err) {
      adicionarLog(`ERRO ao cancelar: ${err}`);
    } finally {
      adicionarLog("--- Operação de cancelamento concluída ---");
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
</script>

<FeatureRunner
  titulo="Agendar Desligamento"
  descricao="Agende o desligamento automático do seu computador após um período de tempo. Você pode cancelar o desligamento agendado a qualquer momento."
  textoBotao={agendamentoAtivo
    ? "Desligamento Agendado..."
    : "Agendar Desligamento"}
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
>
  <div class="mb-2">
    <label for="tempo-desligamento" class="font-bold text-text-light mb-2 block"
      >Selecione o tempo para desligar (segundos):</label
    >
    <select
      id="tempo-desligamento"
      bind:value={tempoSelecionado}
      disabled={emExecucao || agendamentoAtivo}
      class="w-full py-2 bg-dark-blue-light text-text-light border border-primary-purple rounded-md text-base focus:outline-none focus:ring-1 focus:ring-primary-purple disabled:opacity-60 disabled:cursor-not-allowed"
    >
      <option value="60">1 minuto (60s)</option>
      <option value="300">5 minutos (300s)</option>
      <option value="600">10 minutos (600s)</option>
      <option value="1800">30 minutos (1800s)</option>
      <option value="3600">1 hora (3600s)</option>
      <option value="7200">2 horas (7200s)</option>
      <option value="18000">5 horas (18000s)</option>
    </select>
  </div>

  {#if agendamentoAtivo}
    <button
      class="px-4 py-2 text-base font-semibold cursor-pointer bg-red-600 text-white border-none rounded-lg transition-all duration-200 hover:brightness-110 mt-2 mx-auto block disabled:opacity-60 disabled:cursor-not-allowed"
      on:click={cancelarDesligamento}
      disabled={emExecucao}
    >
      Cancelar Desligamento Agendado
    </button>
  {/if}
</FeatureRunner>

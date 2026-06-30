<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ExecutarComando } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando limpeza e reinício do Spool de Impressão..."];

    try {
      adicionarLog("Parando o serviço Spooler...");
      const resultStop = await ExecutarComando("net", ["stop", "spooler"]);
      adicionarLog("Resultado: " + resultStop);

      adicionarLog("Aguardando 3 segundos antes de reiniciar...");
      await new Promise((resolve) => setTimeout(resolve, 3000));

      adicionarLog("Reiniciando o serviço Spooler...");
      const resultStart = await ExecutarComando("net", ["start", "spooler"]);
      adicionarLog("Resultado: " + resultStart);

      adicionarLog("Problemas de impressoras travadas devem estar resolvidos.");
      adicionarLog("--- Operação concluída ---");
    } catch (err) {
      adicionarLog(`ERRO: ${err}`);
      adicionarLog("--- Operação finalizada com erro ---");
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
  titulo="Limpar e Reiniciar Spool de Impressão"
  descricao="Para e reinicia o serviço de Spooler de Impressão, útil para resolver problemas de impressoras travadas."
  textoBotao="Executar Limpeza"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
/>

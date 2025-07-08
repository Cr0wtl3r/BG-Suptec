<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ExecutarComando } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando limpeza do cache DNS..."];

    try {
      adicionarLog("Executando comando: ipconfig /flushdns");
      const resultado = await ExecutarComando("ipconfig", ["/flushdns"]);
      adicionarLog("Resultado: " + resultado);
      adicionarLog("Cache DNS limpo com sucesso!");
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
  titulo="Limpar Cache DNS"
  descricao="Executa o comando 'ipconfig /flushdns' para limpar o cache de resolução de DNS local."
  textoBotao="Executar Limpeza"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
/>

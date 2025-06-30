<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { ExecutarComando } from '../../../wailsjs/go/main/App';
  import FeatureRunner from './FeatureRunner.svelte';

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando desativação da hibernação do Windows..."];

    try {
      adicionarLog("Executando comando: powercfg -h off");
      const resultado = await ExecutarComando("powercfg", ["-h", "off"]);
      adicionarLog("Resultado: " + resultado);
      adicionarLog("Hibernação desativada com sucesso!");
      adicionarLog("O arquivo hiberfil.sys foi removido, liberando espaço em disco.");
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
  titulo="Desativar Hibernação do Windows"
  descricao="Desativa o recurso de hibernação e remove o arquivo hiberfil.sys, liberando espaço em disco."
  textoBotao="Desativar Hibernação"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch('voltar')}
/>
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AtivarOffice } from "../../../wailsjs/go/main/App";

  import LogPanel from "../shared/LogPanel.svelte";
  import FeatureContainer from "../shared/FeatureContainer.svelte";
  import BotaoVoltar from "../shared/BotaoVoltar.svelte";

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let versaoSelecionada = "2024";

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando ativação..."];
    try {
      await AtivarOffice(versaoSelecionada);
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
    const eventName = "log:ativacao:office";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureContainer titulo="Ativação do Office (180 Dias)">
  <p>
    Esta função fechará todos os aplicativos do Office (Word, Excel, etc.) e
    tentará a ativação via KMS.
  </p>

  <div class="config-section">
    <label for="versao-office">Selecione a versão do Office:</label>
    <select
      id="versao-office"
      bind:value={versaoSelecionada}
      disabled={emExecucao}
    >
      <option value="2024">Office 2024</option>
      <option value="2021">Office 2021</option>
      <option value="2016">Office 2016</option>
    </select>
  </div>

  <button class="btn-executar" on:click={iniciar} disabled={emExecucao}>
    {emExecucao ? "Ativando..." : "Iniciar Ativação do Office"}
  </button>

  <LogPanel {logLines} />
  <BotaoVoltar on:click={() => dispatch("voltar")} />
</FeatureContainer>

<style>
  .config-section {
    margin: 25px 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
</style>

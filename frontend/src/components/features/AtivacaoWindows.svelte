<script lang="ts">
  import { onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AtivarWindows } from "../../../wailsjs/go/main/App";

  import LogPanel from "../shared/LogPanel.svelte";
  import FeatureContainer from "../shared/FeatureContainer.svelte";

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let versaoSelecionada = "pro";

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando ativação..."];
    try {
      await AtivarWindows(versaoSelecionada);
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
    const eventName = "log:ativacao:windows";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureContainer titulo="Ativação do Windows (180 Dias)">
  <p>
    Esta função tentará ativar sua versão do Windows usando servidores KMS
    públicos. Requer privilégios de Administrador.
  </p>

  <div class="config-section">
    <label for="versao-windows">Selecione a versão do Windows:</label>
    <select
      id="versao-windows"
      bind:value={versaoSelecionada}
      disabled={emExecucao}
    >
      <option value="pro">Windows 10/11 Pro</option>
      <option value="home">Windows 10/11 Home</option>
      <option value="education">Windows 10/11 Education</option>
      <option value="enterprise">Windows 10/11 Enterprise</option>
    </select>
  </div>

  <button class="btn-executar" on:click={iniciar} disabled={emExecucao}>
    {emExecucao ? "Ativando..." : "Iniciar Ativação do Windows"}
  </button>

  <LogPanel {logLines} />
</FeatureContainer>

<style>
  .config-section {
    margin: 25px 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
</style>

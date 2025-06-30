<script lang="ts">
  import { onMount } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AtivarWindows } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte"; // Usamos nosso novo runner!

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let versaoSelecionada = "pro";

  // A função 'iniciar' agora é chamada pelo evento 'start' do FeatureRunner
  async function iniciar() {
    emExecucao = true;
    logLines = [];
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

<FeatureRunner
  titulo="Ativação do Windows (180 Dias)"
  descricao="Esta função tentará ativar sua versão do Windows usando servidores KMS públicos. Requer privilégios de Administrador."
  textoBotao="Iniciar Ativação do Windows"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar
>
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
</FeatureRunner>

<style>
  .config-section {
    margin-bottom: 25px;
  }
</style>

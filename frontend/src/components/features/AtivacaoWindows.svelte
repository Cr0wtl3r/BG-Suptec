<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { EventsOn, EventsOff, EventsOnce } from "../../../wailsjs/runtime";
  import { AtivarWindows } from "../../../wailsjs/go/main/App";

  // Importando os blocos de construção diretamente
  import FeatureContainer from "../shared/FeatureContainer.svelte";
  import LogPanel from "../shared/LogPanel.svelte";
  import BotaoVoltar from "../shared/BotaoVoltar.svelte";

  const dispatch = createEventDispatcher();
  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let versaoSelecionada = "pro";

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando ativação do Windows..."];

    const promessaDeConclusao = new Promise<void>((resolve) => {
      EventsOnce("ativacao:windows:finalizado", () => {
        resolve();
      });
    });

    try {
      AtivarWindows(versaoSelecionada);
      await promessaDeConclusao;
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
    } finally {
      emExecucao = false;
    }
  }

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    logLines = [...logLines, `[${timestamp}] ${mensagem}`];
  }

  onMount(() => {
    const eventName = "log:ativacao:windows";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureContainer titulo="Ativação do Windows">
  <div class="flex-grow flex flex-col min-h-0">
    <p class="flex-shrink-0 opacity-90 text-center mb-6">
      Esta função tentará ativar sua versão do Windows usando servidores KMS
      públicos. Requer privilégios de Administrador.
    </p>

    <div class="flex-shrink-0 max-w-xl mx-auto w-full">
      <div class="flex items-end gap-4 text-left">
        <div class="flex-grow">
          <label
            for="versao-windows"
            class="block font-bold text-text-light mb-2"
            >Selecione a versão:</label
          >
          <select
            id="versao-windows"
            bind:value={versaoSelecionada}
            disabled={emExecucao}
            class="w-full p-3 bg-dark-blue-light text-text-light border border-primary-purple rounded-md text-base focus:outline-none focus:ring-1 focus:ring-primary-purple disabled:opacity-60 disabled:cursor-not-allowed"
          >
            <option value="pro">Windows 10/11 Pro</option>
            <option value="home">Windows 10/11 Home</option>
            <option value="education">Windows 10/11 Education</option>
            <option value="enterprise">Windows 10/11 Enterprise</option>
          </select>
        </div>
        <button
          class="flex-shrink-0 px-6 py-3 text-lg font-bold cursor-pointer bg-accent-orange text-dark-blue-bg border-none rounded-lg transition-all duration-200 hover:brightness-110 disabled:bg-gray-600 disabled:cursor-not-allowed"
          on:click={iniciar}
          disabled={emExecucao}
        >
          {emExecucao ? "Ativando..." : "Ativar"}
        </button>
      </div>
    </div>

    <LogPanel {logLines} />
  </div>

  <div class="flex-shrink-0 pt-4">
    <BotaoVoltar on:click={() => dispatch("voltar")} />
  </div>
</FeatureContainer>

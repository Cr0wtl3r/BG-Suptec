<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { EventsOn, EventsOff, EventsOnce } from "../../../wailsjs/runtime";
  import { AtivarOffice } from "../../../wailsjs/go/main/App";

  // Importando os blocos de construção diretamente
  import FeatureContainer from "../shared/FeatureContainer.svelte";
  import LogPanel from "../shared/LogPanel.svelte";
  import BotaoVoltar from "../shared/BotaoVoltar.svelte";

  const dispatch = createEventDispatcher();
  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let versaoSelecionada = "2024";

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando ativação do Office..."];

    const promessaDeConclusao = new Promise<void>((resolve) => {
      EventsOnce("ativacao:office:finalizado", () => {
        resolve();
      });
    });

    try {
      AtivarOffice(versaoSelecionada);
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
    const eventName = "log:ativacao:office";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureContainer titulo="Ativação do Microsoft Office">
  <div class="flex-grow flex flex-col min-h-0">
    <p class="flex-shrink-0 opacity-90 text-center mb-6">
      Esta função fechará todos os aplicativos do Office (Word, Excel, etc.) e
      realizará a ativação utilizando servidores KMS públicos.
    </p>

    <div class="flex-shrink-0 max-w-xl mx-auto w-full">
      <div class="flex items-end gap-4 text-left">
        <div class="flex-grow">
          <label
            for="versao-office"
            class="block font-bold text-text-light mb-2"
            >Selecione a versão:</label
          >
          <select
            id="versao-office"
            bind:value={versaoSelecionada}
            disabled={emExecucao}
            class="w-full p-3 bg-dark-blue-light text-text-light border border-primary-purple rounded-md text-base focus:outline-none focus:ring-1 focus:ring-primary-purple disabled:opacity-60 disabled:cursor-not-allowed"
          >
            <option value="2024">Office 2024</option>
            <option value="2021">Office 2021</option>
            <option value="2016">Office 2016</option>
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

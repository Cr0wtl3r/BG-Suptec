<script lang="ts">
  import { tick } from "svelte";

  export let logLines: string[] = [];

  // MUDANÇA CRÍTICA: Adicionado o segundo parâmetro 'logs' na assinatura da função.
  // Embora não seja usado diretamente dentro da função, ele é a "cola" que diz
  // ao Svelte para chamar a função 'update' sempre que 'logLines' mudar.
  function autoscroll(node: HTMLElement, logs: string[]) {
    async function scrollToBottom() {
      await tick();
      node.scrollTop = node.scrollHeight;
    }

    scrollToBottom();

    return {
      // Agora, com o parâmetro na função principal, o Svelte vai chamar este
      // 'update' corretamente toda vez que a propriedade 'logLines' for atualizada.
      update() {
        scrollToBottom();
      },
    };
  }
</script>

<div
  class="bg-dark-blue-bg bg-opacity-50 rounded-lg mt-5 p-4 overflow-y-auto text-sm border border-gray-700 scroll-smooth flex-grow min-h-0"
  use:autoscroll={logLines}
>
  <pre
    class="m-0 whitespace-pre-wrap break-words font-mono text-text-light">{logLines.join(
      "\n",
    )}</pre>
</div>

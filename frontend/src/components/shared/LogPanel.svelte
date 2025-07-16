<script lang="ts">
  export let logLines: string[] = [];

  function autoscroll(node: HTMLElement) {
    function scrollToBottom() {
      // Pequeno timeout para garantir que o DOM foi atualizado antes do scroll
      setTimeout(() => {
        node.scrollTop = node.scrollHeight;
      }, 0);
    }

    scrollToBottom();

    return {
      update() {
        scrollToBottom();
      },
    };
  }
</script>

<div
  class="bg-dark-blue-bg bg-opacity-50 rounded-lg mt-5 p-4 overflow-y-auto text-sm border border-gray-700 scroll-smooth flex-grow min-h-0"
  use:autoscroll
>
  <pre
    class="m-0 whitespace-pre-wrap break-words font-mono text-text-light">{logLines.join(
      "\n",
    )}</pre>
</div>

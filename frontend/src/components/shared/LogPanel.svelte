<script lang="ts">
  export let logLines: string[] = [];
  function autoscroll(node: HTMLElement, logs: string[]) {
    function scrollToBottom() {
      setTimeout(() => {
        node.scrollTop = node.scrollHeight;
      }, 0);
    }
    scrollToBottom();

    return {
      update(newLogs) {
        scrollToBottom();
      },
    };
  }
</script>

<div
  class="bg-dark-blue-bg bg-opacity-50 rounded-lg mt-5 p-4 h-72 overflow-y-auto text-sm border border-gray-700 scroll-smooth"
  use:autoscroll={logLines}
>
  <pre
    class="m-0 whitespace-pre-wrap break-words font-mono text-text-light">{logLines.join(
      "\n",
    )}</pre>
</div>

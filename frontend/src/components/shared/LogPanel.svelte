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

<div class="log-panel" use:autoscroll={logLines}>
  <pre>{logLines.join("\n")}</pre>
</div>

<style>
  .log-panel {
    background-color: rgba(4, 7, 53, 0.534);
    border-radius: 8px;
    margin-top: 20px;
    padding: 15px;
    height: 300px;
    overflow-y: auto;
    font-family: "Courier New", Courier, monospace;
    font-size: 0.9rem;
    border: 1px solid var(--bg-light);
    scroll-behavior: smooth;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>

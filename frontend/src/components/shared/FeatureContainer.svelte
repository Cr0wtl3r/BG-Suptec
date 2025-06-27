<script lang="ts">
  export let titulo: string;
</script>

<div class="feature-container">
  <h2>{titulo}</h2>

  <slot></slot>
</div>

<style>
  .feature-container {
    width: 100%;
    max-width: 800px;
    padding: 25px;
    background-color: rgba(25, 28, 89, 0.4);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(240, 240, 240, 0.3);
    border-radius: 12px;
    color: var(--text-light);
    animation: fadeIn 0.5s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  :global(.feature-container h2) {
    margin-top: 0;
    color: var(--accent-orange);
  }
</style>

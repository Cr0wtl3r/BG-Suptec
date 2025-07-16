<script lang="ts">
  export let titulo: string;
</script>

<div
  class="w-full h-full p-6 bg-dark-blue-light bg-opacity-35 backdrop-blur-md border border-gray-700 rounded-xl text-text-light animate-fadeIn flex flex-col"
>
  <h2 class="flex-shrink-0 mt-0 text-2xl font-bold text-accent-orange mb-4">
    {titulo}
  </h2>
  <div class="flex-grow flex flex-col min-h-0">
    <slot></slot>
  </div>
</div>

<style>
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
  .animate-fadeIn {
    animation: fadeIn 0.5s ease-out;
  }
</style>

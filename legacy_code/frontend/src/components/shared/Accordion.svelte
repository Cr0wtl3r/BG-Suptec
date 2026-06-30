<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let title: string;
  export let isOpen: boolean = false;

  const dispatch = createEventDispatcher();

  function toggleAccordion() {
    isOpen = !isOpen;
    dispatch("toggle", isOpen);
  }
</script>

<div class="mb-4 bg-dark-blue-light rounded-lg shadow-md overflow-hidden">
  <button
    class="flex justify-between items-center w-full p-4 text-left text-lg font-semibold bg-primary-purple hover:bg-primary-purple-dark text-white focus:outline-none transition-colors duration-200"
    on:click={toggleAccordion}
    aria-expanded={isOpen}
  >
    {title}
    <svg
      class="w-5 h-5 transition-transform duration-200"
      class:rotate-90={isOpen}
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M9 5l7 7-7 7"
      />
    </svg>
  </button>
  <div
    class="bg-dark-blue-light transition-all duration-300 ease-in-out overflow-hidden"
    class:max-h-0={!isOpen}
    class:max-h-screen={isOpen}
    class:py-3={isOpen}
    class:px-4={isOpen}
  >
    <div class="py-3 px-4">
      <slot></slot>
    </div>
  </div>
</div>

<style>
  .max-h-0 {
    max-height: 0 !important; /* Adiciona !important para garantir precedência */
    padding-top: 0 !important; /* Remove padding quando fechado */
    padding-bottom: 0 !important;
  }
  .max-h-screen {
    max-height: 100vh;
  }
</style>

<script lang="ts">
  import Login from "./components/Login.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import MainView from "./components/MainView.svelte";
  import PainelInformacoes from "./components/features/PainelInformacoes.svelte";

  let logado = false;
  let menuAberto = false;
  let visaoAtual = "Painel de Informações";

  const modulos = [
    {
      nome: "Info Rápida do Sistema",
      funcionalidades: ["Painel de Informações"],
    },
    {
      nome: "Ativação",
      funcionalidades: ["Office - Ativação", "Windows - Ativação"],
    },
    {
      nome: "Manutenção e Limpeza",
      funcionalidades: [
        "Limpeza de Temporários",
        "Limpeza Completa do PC",
        "Limpa cache DNS",
        "Limpa e Reinicia Spool de Impressão",
        "Desativa a Hibernação do Windows",
        "Limpeza de Drivers Antigos",
      ],
    },
    {
      nome: "Reparos e Soluções",
      funcionalidades: [
        "Corrige Compartilhamento de Rede",
        "Corrige Problemas de Impressoras",
        "Corrige Indexação e Busca",
        "Solução de Problemas do Windows",
        "Conclusão de Formatação",
        "Ajustar Hora da Formatação",
      ],
    },
    {
      nome: "Segurança e Proteção",
      funcionalidades: [
        "Ativar Windows Defender",
        "Desativar Windows Defender",
        "Ativar Proteção do Sistema",
        "Desativar Smart Screen",
      ],
    },
    {
      nome: "Personalização e Sistema",
      funcionalidades: [
        "Ajuste de Políticas do Windows 11",
        "Restaurar Menu de Contexto (Win 11)",
        "Restaurar Visualizador de Fotos",
        "Liberar Instalação (Win 11)",
        "Ativar Gpedit.msc (Home)",
        "Alterar Layout do Teclado",
      ],
    },
    {
      nome: "Serviços e Integrações",
      funcionalidades: [
        "Desativar Windows Update",
        "Reativar Windows Update",
        "Desativar Integração do OneDrive",
        "Reativar Integração do OneDrive",
      ],
    },
    {
      nome: "Ferramentas de Rede",
      funcionalidades: ["Liberar e Renovar IP", "Testar Conexão (Ping)"],
    },
    {
      nome: "Ferramentas de Disco",
      funcionalidades: ["Converter MBR para GPT"],
    },
    {
      nome: "Atalhos de Admin",
      funcionalidades: [
        "Gerenciador de Dispositivos",
        "Painel de Controle",
        "Gerenciamento de Disco",
      ],
    },
    {
      nome: "Gerenciador de Energia",
      funcionalidades: [
        "Reiniciar em Modo de Segurança",
        "Agendar Desligamento",
      ],
    },
  ];

  function handleNavigate(event: CustomEvent) {
    visaoAtual = event.detail;
    menuAberto = false;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      menuAberto = !menuAberto;
    }
  }
</script>

<main
  class="relative w-screen min-h-screen flex flex-col justify-center items-center bg-dark-blue-bg text-text-light **overflow-x-hidden**"
  style="background-image: url('/background.jpg'); background-size: cover; background-position: center center; background-repeat: no-repeat; background-attachment: fixed;;"
>
  <div class="absolute inset-0 bg-black bg-opacity-40"></div>

  <button
    class="absolute top-5 right-5 p-2 z-50 flex flex-col justify-between w-8 h-8 bg-transparent border-none outline-none cursor-pointer hover:opacity-80 focus:outline-2 focus:outline-white focus:outline-offset-2 gap-1.5"
    on:click={() => (menuAberto = !menuAberto)}
    on:keydown={handleKeyDown}
    tabindex="0"
    aria-label="Abrir menu"
  >
    <div
      class="w-full h-0.5 bg-white rounded transition-all duration-300"
    ></div>
    <div
      class="w-full h-0.5 bg-white rounded transition-all duration-300"
    ></div>
    <div
      class="w-full h-0.5 bg-white rounded transition-all duration-300"
    ></div>
  </button>

  {#if menuAberto}
    <Sidebar
      {logado}
      {modulos}
      on:close={() => (menuAberto = false)}
      on:navigate={handleNavigate}
    />
  {/if}

  {#if !logado}
    <Login
      on:loginsucesso={() => {
        logado = true;
        visaoAtual = "Painel de Informações";
      }}
    />
  {:else if visaoAtual === "Painel de Informações"}
    <PainelInformacoes {modulos} on:navigate={handleNavigate} />
  {:else}
    <MainView bind:visao={visaoAtual} on:navigate={handleNavigate} />
  {/if}
</main>

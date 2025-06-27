<script lang="ts">
  import "./app.css";
  import Login from "./components/Login.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import MainView from "./components/MainView.svelte";

  let logado = false;
  let menuAberto = false;
  let visaoAtual = "home";

  const modulos = [
    {
      nome: "Info Rápida do Sistema",

      funcionalidades: ["Painel de Informações"],
    },

    {
      nome: "Ativação",

      funcionalidades: [
        "Office - Ativação 180 dias",

        "Windows - Ativação 180 dias",
      ],
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

  function handleNavigate(event) {
    visaoAtual = event.detail;
    menuAberto = false;
  }
  function handleMenuKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      menuAberto = !menuAberto;
    }
  }
</script>

<main class="container">
  <div
    class="hamburger-menu"
    role="button"
    tabindex="0"
    on:click={() => (menuAberto = !menuAberto)}
    on:keydown={handleMenuKeydown}
  >
    <svg
      width="32"
      height="32"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M4 6H20M4 12H20M4 18H20"
        stroke="var(--text-light)"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </div>

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
  {:else}
    <MainView visao={visaoAtual} {modulos} on:navigate={handleNavigate} />
  {/if}
</main>

<style>
  .container {
    background-image: url("/background.jpg");
    background-size: cover;
    background-position: center center;
    background-repeat: no-repeat;
    background-attachment: fixed;
    background-color: var(--bg-dark);
    width: 100vw;
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
    position: relative;
  }

  .container::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.3);
  }

  .hamburger-menu {
    position: absolute;
    top: 20px;
    right: 20px;
    cursor: pointer;
    padding: 10px;
    z-index: 100; /* Z-index alto para garantir que fique sempre acima do sidebar */
  }

  .hamburger-menu:hover {
    opacity: 0.8;
  }
</style>

interface Modulo {
  nome: string;
  funcionalidades: string[];
}

interface SidebarProps {
  logado: boolean;
  modulos: Modulo[];
  onNavigate: (item: string) => void;
  onClose: () => void;
}

function Sidebar({ logado, modulos, onNavigate, onClose }: SidebarProps) {
  function navegar(item: string) {
    if (!logado) return;
    onNavigate(item);
  }

  function handleBackdropKeyDown(event: React.KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      onClose();
    }
  }

  return (
    <>
      <div
        className="fixed inset-0 z-40 bg-black/50"
        role="button"
        tabIndex={0}
        onClick={onClose}
        onKeyDown={handleBackdropKeyDown}
      />

      <aside className="animate-slideIn fixed top-0 right-0 z-50 h-screen w-80 overflow-y-auto border-l border-gray-700 bg-dark-blue-light/35 shadow-xl backdrop-blur-md">
        <div className="p-5">
          <h2 className="mb-8 text-center text-2xl font-bold text-accent-orange">
            Módulos
          </h2>
          <nav>
            {modulos.map((modulo) => (
              <div className="mb-6" key={modulo.nome}>
                <h3 className="mt-0 mb-2 border-b-2 border-structural-purple pb-1 text-lg font-semibold text-text-light">
                  {modulo.nome}
                </h3>
                <ul>
                  {modulo.funcionalidades.map((item) => (
                    <li key={item}>
                      <button
                        className="block w-full cursor-pointer rounded-md border-none bg-transparent px-3 py-2 text-left text-base text-text-light transition-colors duration-200 hover:bg-dark-blue-bg disabled:cursor-not-allowed disabled:text-gray-500 disabled:opacity-60"
                        disabled={!logado}
                        onClick={() => navegar(item)}
                      >
                        {item}
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </nav>
        </div>
      </aside>
    </>
  );
}

export default Sidebar;

type ModalTipo = "sucesso" | "erro" | "aviso";

interface ModalProps {
  tipo?: ModalTipo;
  titulo: string;
  mensagem: string;
  textoConfirmar?: string | null;
  textoCancelar?: string | null;
  onConfirmar?: () => void;
  onCancelar: () => void;
}

const cores: Record<ModalTipo, string> = {
  sucesso: "bg-green-700/20 border-green-700/30 text-green-500",
  erro: "bg-red-700/20 border-red-700/30 text-red-500",
  aviso: "bg-yellow-600/20 border-yellow-600/30 text-yellow-400",
};

function ModalIcon({ tipo }: { tipo: ModalTipo }) {
  if (tipo === "sucesso") {
    return (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={3}>
        <polyline points="20,6 9,17 4,12" />
      </svg>
    );
  }
  if (tipo === "erro") {
    return (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={3}>
        <circle cx="12" cy="12" r="10" />
        <line x1="15" y1="9" x2="9" y2="15" />
        <line x1="9" y1="9" x2="15" y2="15" />
      </svg>
    );
  }
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
      <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
      <line x1="12" y1="9" x2="12" y2="13" />
      <line x1="12" y1="17" x2="12.01" y2="17" />
    </svg>
  );
}

function Modal({
  tipo = "aviso",
  titulo,
  mensagem,
  textoConfirmar = null,
  textoCancelar = "Fechar",
  onConfirmar,
  onCancelar,
}: ModalProps) {
  return (
    <div
      className="fixed inset-0 z-[1000] flex animate-[modalFadeIn_0.2s_ease-out] items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onCancelar}
      onKeyDown={(e) => e.key === "Escape" && onCancelar()}
      role="button"
      tabIndex={0}
    >
      <div
        className="w-11/12 max-w-lg animate-[modalSlideIn_0.3s_ease-out] overflow-hidden rounded-xl border border-gray-700 bg-dark-blue-light/95 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
      >
        <div className={`flex items-center gap-4 border-b border-gray-800 p-5 ${cores[tipo]}`}>
          <div className="flex-shrink-0">
            <ModalIcon tipo={tipo} />
          </div>
          <h3 className="m-0 flex-grow text-lg font-semibold text-text-light">{titulo}</h3>
        </div>
        <div className="p-5">
          <p className="m-0 text-base leading-normal text-gray-200">{mensagem}</p>
        </div>
        <div className="flex justify-end gap-3 px-5 pb-5">
          {textoCancelar && (
            <button
              type="button"
              className="cursor-pointer rounded-md border border-gray-600 bg-dark-blue-bg px-5 py-2.5 text-base font-medium text-white transition-all duration-200 hover:bg-gray-700"
              onClick={onCancelar}
            >
              {textoCancelar}
            </button>
          )}
          {textoConfirmar && (
            <button
              type="button"
              className="cursor-pointer rounded-md border-none bg-accent-orange px-5 py-2.5 text-base font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110"
              onClick={onConfirmar}
            >
              {textoConfirmar}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export default Modal;

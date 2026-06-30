import { useState } from "react";

interface EditableFieldProps {
  label: string;
  value: string;
  disabled?: boolean;
  onSave: (value: string) => void;
}

// Note: DESIGN.md explicitly corrects the legacy `border-l-4` side-stripe
// treatment here — full subtle border (gray-700, structural-purple on
// edit) + pencil icon communicates "editable", not a colored stripe.
function EditableField({ label, value, disabled = false, onSave }: EditableFieldProps) {
  const [editando, setEditando] = useState(false);
  const [valorEditavel, setValorEditavel] = useState(value);

  function ativarEdicao() {
    if (disabled) return;
    setValorEditavel(value);
    setEditando(true);
  }

  function salvar() {
    onSave(valorEditavel);
    setEditando(false);
  }

  function cancelar() {
    setEditando(false);
  }

  return (
    <div
      className={`flex flex-col rounded-md border bg-dark-blue-bg p-3 transition-colors duration-200 ${
        editando ? "border-structural-purple" : "border-gray-700"
      }`}
    >
      <span className="mb-1 block text-center text-xs uppercase opacity-70">{label}</span>

      {!editando ? (
        <div className="relative flex min-h-[28px] items-center justify-center">
          <span className="break-all px-5 text-center text-sm font-semibold">{value}</span>
          <button
            type="button"
            aria-label="Editar campo"
            title="Editar"
            className="absolute right-0 top-1/2 -translate-y-1/2 cursor-pointer border-none bg-transparent p-0.5 text-text-light opacity-60 transition-opacity duration-200 hover:text-accent-orange hover:opacity-100 disabled:cursor-not-allowed disabled:opacity-20"
            onClick={ativarEdicao}
            disabled={disabled}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={2}
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
            </svg>
          </button>
        </div>
      ) : (
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={valorEditavel}
            onChange={(e) => setValorEditavel(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") salvar();
              if (e.key === "Escape") cancelar();
            }}
            className="flex-grow rounded border border-structural-purple bg-dark-blue-light p-1.5 text-center text-sm font-semibold text-text-light"
            autoFocus
          />
          <button
            type="button"
            aria-label="Salvar edição"
            title="Salvar"
            className="cursor-pointer border-none bg-transparent p-0.5 text-text-light opacity-60 transition-opacity duration-200 hover:text-green-500 hover:opacity-100"
            onClick={salvar}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={3}
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </button>
          <button
            type="button"
            aria-label="Cancelar edição"
            title="Cancelar"
            className="cursor-pointer border-none bg-transparent p-0.5 text-text-light opacity-60 transition-opacity duration-200 hover:text-red-500 hover:opacity-100"
            onClick={cancelar}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={3}
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      )}
    </div>
  );
}

export default EditableField;

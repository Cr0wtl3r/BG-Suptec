import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import BotaoVoltar from "../shared/BotaoVoltar";
import TecladoABNT2 from "../shared/teclados/TecladoABNT2";
import TecladoUS from "../shared/teclados/TecladoUS";
import TecladoES from "../shared/teclados/TecladoES";

interface TecladoInfo {
  id: string;
  nome: string;
  tagIdioma: string;
}

interface AlterarLayoutTecladoProps {
  onVoltar: () => void;
}

function previewParaLayout(layoutSelecionado: string | undefined) {
  if (!layoutSelecionado) return null;
  if (layoutSelecionado.includes("0416")) return TecladoABNT2;
  if (layoutSelecionado.includes("0816")) return TecladoABNT2;
  if (layoutSelecionado.includes("0409")) return TecladoUS;
  if (layoutSelecionado.includes("0c0a") || layoutSelecionado.includes("080a")) return TecladoES;
  return null;
}

function AlterarLayoutTeclado({ onVoltar }: AlterarLayoutTecladoProps) {
  const [todosLayouts, setTodosLayouts] = useState<TecladoInfo[]>([]);
  const [nomeLayoutAtual, setNomeLayoutAtual] = useState("Carregando...");
  const [idLayoutAtivo, setIdLayoutAtivo] = useState("");
  const [layoutSelecionado, setLayoutSelecionado] = useState<string | undefined>(undefined);
  const [erro, setErro] = useState("");
  const [salvando, setSalvando] = useState(false);
  const [mensagemSucesso, setMensagemSucesso] = useState("");

  useEffect(() => {
    (async () => {
      try {
        const layouts = await invoke<TecladoInfo[]>("obter_layouts_teclado");
        const ativoId = await invoke<string>("obter_layout_ativo");
        setTodosLayouts(layouts);
        setIdLayoutAtivo(ativoId);
        const ativo = layouts.find((l) => l.id === ativoId);
        if (ativo) {
          setNomeLayoutAtual(ativo.nome);
          setLayoutSelecionado(ativo.id);
        } else {
          setNomeLayoutAtual(`Não reconhecido (${ativoId})`);
          if (layouts.length > 0) {
            setLayoutSelecionado(layouts[0].id);
          }
        }
      } catch (e) {
        setErro(`Erro ao carregar layouts: ${e}`);
        setNomeLayoutAtual("Erro na detecção");
      }
    })();
  }, []);

  async function aplicarLayout() {
    if (!layoutSelecionado) return;
    const layoutParaAplicar = todosLayouts.find((l) => l.id === layoutSelecionado);
    if (!layoutParaAplicar) {
      setErro("Erro interno: Layout selecionado não encontrado na lista.");
      return;
    }

    setSalvando(true);
    setMensagemSucesso("");
    setErro("");
    try {
      await invoke("alterar_layout_teclado", { tagIdioma: layoutParaAplicar.tagIdioma });
      setMensagemSucesso("Layout do teclado alterado com sucesso!");

      await new Promise((resolve) => setTimeout(resolve, 1000));

      const ativoId = await invoke<string>("obter_layout_ativo");
      setIdLayoutAtivo(ativoId);
      const ativo = todosLayouts.find((l) => l.id === ativoId);
      setNomeLayoutAtual(ativo ? ativo.nome : `Não reconhecido (${ativoId})`);
    } catch (e) {
      setErro(`Falha ao aplicar layout: ${e}`);
    } finally {
      setSalvando(false);
    }
  }

  const TecladoComponente = previewParaLayout(layoutSelecionado);

  return (
    <FeatureContainer titulo="Alterar Layout do Teclado">
      <div className="min-h-0 flex-grow overflow-y-auto pr-2">
        <div className="space-y-6">
          <div className="rounded-lg bg-dark-blue-bg/50 p-1 text-center">
            <h3 className="mb-1 text-sm uppercase opacity-70">
              Layout do Teclado Ativo no Sistema
            </h3>
            <p className="text-xl font-bold text-accent-orange">{nomeLayoutAtual}</p>
          </div>

          <div>
            <label htmlFor="layout-select" className="mb-2 block font-bold">
              Selecione um layout para visualizar ou aplicar:
            </label>
            <div className="flex flex-col gap-2 sm:flex-row">
              <select
                id="layout-select"
                className="w-full flex-grow rounded-md border border-structural-purple bg-dark-blue-light p-3 text-base text-text-light focus:outline-none focus:ring-1 focus:ring-structural-purple"
                value={layoutSelecionado ?? ""}
                onChange={(e) => setLayoutSelecionado(e.target.value)}
                disabled={salvando || todosLayouts.length === 0}
              >
                {todosLayouts.map((layout) => (
                  <option key={layout.id} value={layout.id}>
                    {layout.nome}
                    {layout.id === idLayoutAtivo ? " (Atual)" : ""}
                  </option>
                ))}
              </select>
              <button
                className="cursor-pointer rounded-lg border-none bg-accent-orange px-6 py-3 text-lg font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
                onClick={aplicarLayout}
                disabled={salvando || !layoutSelecionado || layoutSelecionado === idLayoutAtivo}
              >
                {salvando ? "Aplicando..." : "Aplicar"}
              </button>
            </div>
            {erro && <p className="mt-2 text-center text-red-500">{erro}</p>}
            {mensagemSucesso && (
              <p className="mt-2 text-center text-green-400">{mensagemSucesso}</p>
            )}
          </div>

          <div className="grid items-center gap-6 md:grid-cols-2">
            <div className="w-full">
              <label htmlFor="test-area" className="mb-2 block font-bold">
                Teste sua digitação aqui:
              </label>
              <textarea
                id="test-area"
                rows={5}
                className="w-full rounded-lg border border-structural-purple/50 bg-dark-blue-bg/70 p-3 text-base focus:outline-none focus:ring-1 focus:ring-structural-purple"
                placeholder="Teste teclas como ´ ` ~ ^ ç ; / . ,"
              />
            </div>

            <div className="w-full">
              <h3 className="mb-2 block text-center font-bold">Visualização</h3>
              <div className="flex min-h-[100px] items-center justify-center rounded-lg bg-dark-blue-bg/30 p-3">
                {TecladoComponente ? (
                  <TecladoComponente />
                ) : (
                  <p className="text-center italic opacity-70">
                    Nenhuma visualização disponível para este layout.
                  </p>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
    </FeatureContainer>
  );
}

export default AlterarLayoutTeclado;

import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Accordion from "../shared/Accordion";
import EditableField from "../shared/EditableField";
import Modal from "../shared/Modal";

interface SystemInfo {
  nomeComputador: string;
  versaoWindows: string;
  edicaoWindows: string;
  buildWindows: string;
  processador: string;
  memoriaTotalGB: string;
  enderecoMAC: string;
  enderecoIP: string;
  mascaraRede: string;
  gatewayPadrao: string;
  dnsPrimario: string;
  dnsSecundario: string;
  interfaceAtiva: string;
}

interface Modulo {
  nome: string;
  funcionalidades: string[];
}

interface ModalState {
  visible: boolean;
  tipo?: "sucesso" | "erro" | "aviso";
  titulo?: string;
  mensagem?: string;
  textoConfirmar?: string | null;
  textoCancelar?: string | null;
  onConfirm?: () => void;
}

interface PainelInformacoesProps {
  modulos?: Modulo[];
  onNavigate?: (funcionalidade: string) => void;
}

const LARGURA_MINIMA_ITEM = 180;
const ALTURA_MINIMA_ITEM = 70;
const GAP_DO_GRID = 12;

// InfoCard: read-only counterpart of EditableField, kept visually
// consistent with it (same border treatment, no legacy side-stripe).
function InfoCard({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="rounded-md border border-gray-700 bg-dark-blue-bg p-3 text-center">
      <span className="mb-1 block text-xs uppercase opacity-70">{label}</span>
      <span className="break-all text-sm font-semibold leading-snug">{children}</span>
    </div>
  );
}

function PainelInformacoes({ modulos = [], onNavigate }: PainelInformacoesProps) {
  const [info, setInfo] = useState<SystemInfo | null>(null);
  const [erro, setErro] = useState("");

  const searchInputRef = useRef<HTMLInputElement>(null);
  const gridContainerRef = useRef<HTMLDivElement>(null);
  const [itensPorPagina, setItensPorPagina] = useState(12);

  const [editandoIP, setEditandoIP] = useState(false);
  const [editandoDNS, setEditandoDNS] = useState(false);
  const [tempEnderecoIP, setTempEnderecoIP] = useState("");
  const [tempMascaraRede, setTempMascaraRede] = useState("");
  const [tempGatewayPadrao, setTempGatewayPadrao] = useState("");
  const [tempDNSPrimario, setTempDNSPrimario] = useState("");
  const [tempDNSSecundario, setTempDNSSecundario] = useState("");

  const [salvandoNome, setSalvandoNome] = useState(false);
  const [salvandoIP, setSalvandoIP] = useState(false);
  const [salvandoDNS, setSalvandoDNS] = useState(false);

  const [modal, setModal] = useState<ModalState>({ visible: false });

  const [termoBusca, setTermoBusca] = useState("");
  const [paginaAtual, setPaginaAtual] = useState(1);

  function mostrarModal(novoModal: Omit<ModalState, "visible">) {
    setModal({ ...novoModal, visible: true });
  }

  function fecharModal() {
    setModal({ visible: false });
  }

  async function carregarInformacoes() {
    try {
      const dados = await invoke<SystemInfo>("obter_informacoes_sistema");
      setInfo(dados);
      setErro("");
    } catch (e) {
      setErro(`Erro ao carregar informações: ${e}`);
      setInfo(null);
    }
  }

  useEffect(() => {
    carregarInformacoes();

    function handleGlobalKeydown(event: KeyboardEvent) {
      if (event.ctrlKey || event.altKey || event.metaKey || event.key.length > 1) return;
      const active = document.activeElement;
      if (active?.tagName.toLowerCase() !== "input") {
        searchInputRef.current?.focus();
      }
    }
    window.addEventListener("keydown", handleGlobalKeydown);

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        const novasColunas = Math.max(1, Math.floor(width / (LARGURA_MINIMA_ITEM + GAP_DO_GRID)));
        const novasLinhas = Math.max(1, Math.floor(height / (ALTURA_MINIMA_ITEM + GAP_DO_GRID)));
        setItensPorPagina(novasColunas * novasLinhas);
      }
    });
    if (gridContainerRef.current) {
      observer.observe(gridContainerRef.current);
    }

    return () => {
      window.removeEventListener("keydown", handleGlobalKeydown);
      observer.disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const todasFuncionalidades = useMemo(
    () =>
      modulos
        .flatMap((m) => m.funcionalidades)
        .filter((f) => f !== "Painel de Informações")
        .sort((a, b) => a.localeCompare(b)),
    [modulos],
  );

  const funcionalidadesFiltradas = useMemo(
    () => todasFuncionalidades.filter((f) => f.toLowerCase().includes(termoBusca.toLowerCase())),
    [todasFuncionalidades, termoBusca],
  );

  const totalPaginas = Math.max(1, Math.ceil(funcionalidadesFiltradas.length / itensPorPagina));
  const funcionalidadesPaginadas = funcionalidadesFiltradas.slice(
    (paginaAtual - 1) * itensPorPagina,
    paginaAtual * itensPorPagina,
  );

  useEffect(() => {
    setPaginaAtual(1);
  }, [termoBusca]);

  function iniciarEdicaoIP() {
    setTempEnderecoIP(info?.enderecoIP ?? "");
    setTempMascaraRede(info?.mascaraRede ?? "255.255.255.0");
    setTempGatewayPadrao(info?.gatewayPadrao ?? "");
    setEditandoIP(true);
  }

  function iniciarEdicaoDNS() {
    setTempDNSPrimario(info?.dnsPrimario && info.dnsPrimario !== "N/A" ? info.dnsPrimario : "");
    setTempDNSSecundario(
      info?.dnsSecundario && info.dnsSecundario !== "N/A" ? info.dnsSecundario : "",
    );
    setEditandoDNS(true);
  }

  function cancelarEdicao() {
    setEditandoIP(false);
    setEditandoDNS(false);
  }

  async function handleSalvarNome(novoNomeBruto: string) {
    const novoNome = novoNomeBruto.trim();
    if (!novoNome) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Validação",
        mensagem: "Nome do computador não pode estar vazio!",
      });
      return;
    }

    setSalvandoNome(true);
    try {
      const precisaReiniciar = await invoke<boolean>("alterar_nome_computador", {
        novoNome,
      });
      await carregarInformacoes();

      if (precisaReiniciar) {
        mostrarModal({
          tipo: "sucesso",
          titulo: "Sucesso!",
          mensagem:
            "O nome do computador foi alterado. É necessário reiniciar para que a mudança tenha efeito completo. Deseja reiniciar agora?",
          textoConfirmar: "Reiniciar Agora",
          textoCancelar: "Depois",
          onConfirm: async () => {
            try {
              await invoke("reiniciar_computador");
            } catch (err) {
              mostrarModal({
                tipo: "erro",
                titulo: "Erro",
                mensagem: `Não foi possível reiniciar: ${err}`,
              });
              return;
            }
            fecharModal();
          },
        });
      } else {
        mostrarModal({ tipo: "sucesso", titulo: "Sucesso!", mensagem: "Nome do computador alterado!" });
      }
    } catch (e) {
      mostrarModal({ tipo: "erro", titulo: "Erro", mensagem: `Erro ao alterar nome: ${e}` });
    }
    setSalvandoNome(false);
  }

  async function salvarIP() {
    const ip = tempEnderecoIP.trim();
    const mascara = tempMascaraRede.trim();
    const gateway = tempGatewayPadrao.trim();

    if (ip !== "" && (mascara === "" || gateway === "")) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Validação",
        mensagem:
          "Para definir um IP estático, todos os campos (IP, Máscara e Gateway) devem ser preenchidos.",
      });
      return;
    }

    if (!info?.interfaceAtiva) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Sistema",
        mensagem: "Interface de rede não encontrada. Tente recarregar.",
      });
      return;
    }

    setSalvandoIP(true);
    try {
      await invoke("alterar_ip", {
        interfaceName: info.interfaceAtiva,
        novoIp: ip,
        mascara,
        gateway,
      });
      const mensagem =
        ip === ""
          ? "Configurações de rede definidas para DHCP (dinâmico)!"
          : "Configurações de rede alteradas com sucesso!";

      mostrarModal({ tipo: "sucesso", titulo: "Sucesso!", mensagem });

      await new Promise((resolve) => setTimeout(resolve, 2000));
      await carregarInformacoes();
      setEditandoIP(false);
    } catch (e) {
      mostrarModal({ tipo: "erro", titulo: "Erro", mensagem: `Erro ao alterar IP: ${e}` });
    }
    setSalvandoIP(false);
  }

  async function salvarDNS() {
    const primario = tempDNSPrimario.trim();
    const secundario = tempDNSSecundario.trim();

    if (!info?.interfaceAtiva) {
      mostrarModal({
        tipo: "erro",
        titulo: "Erro de Sistema",
        mensagem: "Interface de rede não encontrada. Tente recarregar.",
      });
      return;
    }

    setSalvandoDNS(true);
    try {
      await invoke("alterar_dns", {
        interfaceName: info.interfaceAtiva,
        dnsPrimario: primario,
        dnsSecundario: secundario,
      });
      const mensagem =
        primario === ""
          ? "Servidores DNS definidos para obter via DHCP (automático)!"
          : "Servidores DNS alterados com sucesso!";

      mostrarModal({ tipo: "sucesso", titulo: "Sucesso!", mensagem });

      await new Promise((resolve) => setTimeout(resolve, 1500));
      await carregarInformacoes();
      setEditandoDNS(false);
    } catch (e) {
      mostrarModal({ tipo: "erro", titulo: "Erro", mensagem: `Erro ao alterar DNS: ${e}` });
    }
    setSalvandoDNS(false);
  }

  function navegarPara(funcionalidade: string) {
    onNavigate?.(funcionalidade);
  }

  return (
    <div className="flex h-full w-full animate-[fadeIn_0.5s_ease-out] flex-col gap-5 overflow-hidden rounded-xl border border-gray-700 bg-dark-blue-light/35 backdrop-blur-md md:flex-row">
      <div className="w-full overflow-y-auto rounded-lg bg-dark-blue-light/25 p-5 md:w-96 md:flex-shrink-0">
        <h2 className="mb-4 text-center text-2xl font-bold text-accent-orange">Painel de Informações</h2>
        {erro ? (
          <p className="text-center text-red-500 opacity-80">{erro}</p>
        ) : !info ? (
          <p className="text-center opacity-80">Carregando...</p>
        ) : (
          <div className="grid grid-cols-1 gap-3">
            <Accordion title="Informações do Sistema" defaultOpen>
              <div className="grid grid-cols-1 gap-3">
                <EditableField
                  label="Computador"
                  value={info.nomeComputador}
                  disabled={salvandoNome}
                  onSave={handleSalvarNome}
                />
                <InfoCard label="RAM">{info.memoriaTotalGB}</InfoCard>
                <InfoCard label="Windows">
                  {info.edicaoWindows} ({info.versaoWindows})
                </InfoCard>
                <InfoCard label="Processador">{info.processador}</InfoCard>
                <InfoCard label="MAC">{info.enderecoMAC}</InfoCard>
              </div>
            </Accordion>

            <Accordion title="Informações de Rede (IPv4)">
              <div className="grid grid-cols-1 gap-3">
                <div className="rounded-md border border-gray-700 bg-dark-blue-bg p-3 text-center">
                  <span className="mb-1 block text-xs uppercase opacity-70">Rede IPv4</span>
                  <div className="relative flex min-h-6 w-full items-start gap-2">
                    {editandoIP ? (
                      <div className="flex w-full flex-col gap-1">
                        <input
                          type="text"
                          value={tempEnderecoIP}
                          onChange={(e) => setTempEnderecoIP(e.target.value)}
                          className="box-border rounded border border-structural-purple bg-dark-blue-light p-1.5 text-center text-sm text-text-light"
                          disabled={salvandoIP}
                          placeholder="IP (deixe em branco para DHCP)"
                        />
                        <input
                          type="text"
                          value={tempMascaraRede}
                          onChange={(e) => setTempMascaraRede(e.target.value)}
                          className="box-border rounded border border-structural-purple bg-dark-blue-light p-1.5 text-center text-sm text-text-light"
                          disabled={salvandoIP}
                          placeholder="Máscara de Sub-rede"
                        />
                        <input
                          type="text"
                          value={tempGatewayPadrao}
                          onChange={(e) => setTempGatewayPadrao(e.target.value)}
                          className="box-border rounded border border-structural-purple bg-dark-blue-light p-1.5 text-center text-sm text-text-light"
                          disabled={salvandoIP}
                          placeholder="Gateway Padrão"
                        />
                        <div className="mt-1 flex items-center justify-end gap-2">
                          <button
                            type="button"
                            aria-label="Salvar IP"
                            onClick={salvarIP}
                            disabled={salvandoIP}
                            title="Salvar"
                            className="cursor-pointer border-none bg-transparent p-0.5 text-text-light opacity-60 transition-opacity duration-200 hover:text-green-500 hover:opacity-100 disabled:cursor-not-allowed disabled:opacity-20"
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
                            aria-label="Cancelar edição de IP"
                            onClick={cancelarEdicao}
                            disabled={salvandoIP}
                            title="Cancelar"
                            className="cursor-pointer border-none bg-transparent p-0.5 text-text-light opacity-60 transition-opacity duration-200 hover:text-red-500 hover:opacity-100 disabled:cursor-not-allowed disabled:opacity-20"
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
                      </div>
                    ) : (
                      <div className="relative flex w-full items-center justify-center">
                        <span className="w-full break-all text-center text-sm font-semibold leading-snug">
                          IP: {info.enderecoIP}
                          <br />
                          Máscara: {info.mascaraRede}
                          <br />
                          Gateway: {info.gatewayPadrao}
                        </span>
                        <button
                          type="button"
                          onClick={iniciarEdicaoIP}
                          className="absolute right-0 top-1/2 -translate-y-1/2 rounded-md bg-transparent p-0.5 text-text-light opacity-60 transition-all duration-200 hover:text-accent-orange hover:opacity-100"
                          aria-label="Editar configurações de rede"
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
                    )}
                  </div>
                </div>
              </div>
            </Accordion>

            <Accordion title="Configurações de DNS">
              <div className="grid grid-cols-1 gap-3">
                <div className="rounded-md border border-gray-700 bg-dark-blue-bg p-3 text-center">
                  <span className="mb-1 block text-xs uppercase opacity-70">DNS</span>
                  <div className="relative flex min-h-6 w-full items-start gap-2">
                    {editandoDNS ? (
                      <div className="flex w-full flex-col gap-1">
                        <input
                          type="text"
                          value={tempDNSPrimario}
                          onChange={(e) => setTempDNSPrimario(e.target.value)}
                          className="box-border rounded border border-structural-purple bg-dark-blue-light p-1.5 text-center text-sm text-text-light"
                          disabled={salvandoDNS}
                          placeholder="DNS (deixe em branco para auto)"
                        />
                        <input
                          type="text"
                          value={tempDNSSecundario}
                          onChange={(e) => setTempDNSSecundario(e.target.value)}
                          className="box-border rounded border border-structural-purple bg-dark-blue-light p-1.5 text-center text-sm text-text-light"
                          disabled={salvandoDNS}
                          placeholder="DNS Secundário (opcional)"
                        />
                        <div className="mt-1 flex items-center justify-end gap-2">
                          <button
                            type="button"
                            aria-label="Salvar DNS"
                            onClick={salvarDNS}
                            disabled={salvandoDNS}
                            title="Salvar"
                            className="cursor-pointer border-none bg-transparent p-0.5 text-text-light opacity-60 transition-opacity duration-200 hover:text-green-500 hover:opacity-100 disabled:cursor-not-allowed disabled:opacity-20"
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
                            aria-label="Cancelar edição de DNS"
                            onClick={cancelarEdicao}
                            disabled={salvandoDNS}
                            title="Cancelar"
                            className="cursor-pointer border-none bg-transparent p-0.5 text-text-light opacity-60 transition-opacity duration-200 hover:text-red-500 hover:opacity-100 disabled:cursor-not-allowed disabled:opacity-20"
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
                      </div>
                    ) : (
                      <div className="relative flex w-full items-center justify-center">
                        <span className="w-full break-all text-center text-sm font-semibold leading-snug">
                          {info.dnsPrimario && info.dnsPrimario !== "N/A" ? (
                            <>
                              Primário: {info.dnsPrimario}
                              {info.dnsSecundario && info.dnsSecundario !== "N/A" && (
                                <>
                                  <br />
                                  Secundário: {info.dnsSecundario}
                                </>
                              )}
                            </>
                          ) : (
                            "Não configurado ou Automático"
                          )}
                        </span>
                        <button
                          type="button"
                          onClick={iniciarEdicaoDNS}
                          className="absolute right-0 top-1/2 -translate-y-1/2 rounded-md bg-transparent p-0.5 text-text-light opacity-60 transition-all duration-200 hover:text-accent-orange hover:opacity-100"
                          aria-label="Editar servidores DNS"
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
                    )}
                  </div>
                </div>
              </div>
            </Accordion>
          </div>
        )}
      </div>

      <div className="flex min-w-0 flex-grow flex-col p-5">
        <h3 className="mb-5 text-xl font-bold text-accent-orange">
          Todas as Ferramentas ({funcionalidadesFiltradas.length})
        </h3>
        <input
          ref={searchInputRef}
          type="search"
          className="mb-4 box-border w-full flex-shrink-0 rounded-lg border border-structural-purple bg-dark-blue-bg/70 p-2.5 text-base text-text-light focus:outline-none focus:ring-1 focus:ring-structural-purple"
          placeholder="Pesquisar ferramenta..."
          value={termoBusca}
          onChange={(e) => setTermoBusca(e.target.value)}
        />
        <div
          ref={gridContainerRef}
          className="grid flex-grow content-start gap-3 overflow-y-auto pb-3 pr-2"
          style={{ gridTemplateColumns: `repeat(auto-fill, minmax(${LARGURA_MINIMA_ITEM}px, 1fr))` }}
        >
          {funcionalidadesPaginadas.map((funcionalidade) => (
            <button
              key={funcionalidade}
              type="button"
              className="cursor-pointer rounded-lg border border-dark-blue-bg bg-dark-blue-light/50 p-4 text-center text-sm font-bold text-text-light transition-all duration-200 hover:-translate-y-px hover:border-accent-orange hover:bg-dark-blue-light"
              onClick={() => navegarPara(funcionalidade)}
            >
              {funcionalidade}
            </button>
          ))}
          {funcionalidadesPaginadas.length === 0 && funcionalidadesFiltradas.length > 0 && (
            <p className="col-span-full text-center opacity-80">Nenhuma ferramenta encontrada nesta página.</p>
          )}
          {funcionalidadesFiltradas.length === 0 && (
            <p className="col-span-full text-center opacity-80">Nenhuma ferramenta encontrada.</p>
          )}
        </div>
        {funcionalidadesFiltradas.length > itensPorPagina && (
          <div className="mt-4 flex flex-shrink-0 items-center justify-center gap-2">
            <button
              type="button"
              className="cursor-pointer rounded-md border border-gray-600 bg-dark-blue-bg px-3 py-1 text-text-light hover:bg-gray-700 disabled:cursor-not-allowed disabled:opacity-50"
              onClick={() => setPaginaAtual((p) => Math.max(1, p - 1))}
              disabled={paginaAtual === 1}
            >
              Anterior
            </button>
            <span className="text-sm text-text-light">
              Página {paginaAtual} de {totalPaginas}
            </span>
            <button
              type="button"
              className="cursor-pointer rounded-md border border-gray-600 bg-dark-blue-bg px-3 py-1 text-text-light hover:bg-gray-700 disabled:cursor-not-allowed disabled:opacity-50"
              onClick={() => setPaginaAtual((p) => Math.min(totalPaginas, p + 1))}
              disabled={paginaAtual === totalPaginas}
            >
              Próxima
            </button>
          </div>
        )}
      </div>

      {modal.visible && (
        <Modal
          tipo={modal.tipo}
          titulo={modal.titulo ?? ""}
          mensagem={modal.mensagem ?? ""}
          textoConfirmar={modal.textoConfirmar}
          textoCancelar={modal.textoCancelar}
          onConfirmar={() => modal.onConfirm?.()}
          onCancelar={fecharModal}
        />
      )}
    </div>
  );
}

export default PainelInformacoes;

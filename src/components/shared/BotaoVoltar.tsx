interface BotaoVoltarProps {
  onClick: () => void;
}

function BotaoVoltar({ onClick }: BotaoVoltarProps) {
  return (
    <button
      className="mx-auto mt-6 block w-fit cursor-pointer rounded-lg border border-structural-purple bg-transparent p-2.5 px-5 text-base font-bold text-text-light transition-all duration-200 hover:bg-structural-purple"
      onClick={onClick}
    >
      &larr; Voltar ao Painel Principal
    </button>
  );
}

export default BotaoVoltar;

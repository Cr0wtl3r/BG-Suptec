import type { ReactNode } from "react";

interface FeatureContainerProps {
  titulo: string;
  children: ReactNode;
}

function FeatureContainer({ titulo, children }: FeatureContainerProps) {
  return (
    <div className="flex h-full w-full animate-[fadeIn_0.5s_ease-out] flex-col rounded-xl border border-gray-700 bg-dark-blue-light/35 p-6 text-text-light backdrop-blur-md">
      <h2 className="mb-4 mt-0 flex-shrink-0 text-2xl font-bold text-accent-orange">{titulo}</h2>
      <div className="flex min-h-0 flex-grow flex-col">{children}</div>
    </div>
  );
}

export default FeatureContainer;

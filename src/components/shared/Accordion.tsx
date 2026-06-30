import { useState, type ReactNode } from "react";

interface AccordionProps {
  title: string;
  defaultOpen?: boolean;
  children: ReactNode;
}

function Accordion({ title, defaultOpen = false, children }: AccordionProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div className="mb-4 overflow-hidden rounded-lg bg-dark-blue-light shadow-md">
      <button
        type="button"
        className="flex w-full items-center justify-between bg-structural-purple p-4 text-left text-lg font-semibold text-text-light transition-colors duration-200 hover:brightness-125 focus:outline-none"
        onClick={() => setIsOpen((open) => !open)}
        aria-expanded={isOpen}
      >
        {title}
        <svg
          className={`h-5 w-5 transition-transform duration-200 ${isOpen ? "rotate-90" : ""}`}
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 5l7 7-7 7"
          />
        </svg>
      </button>
      <div
        className={`overflow-hidden bg-dark-blue-light transition-all duration-300 ease-in-out ${
          isOpen ? "max-h-screen px-4 py-3" : "max-h-0"
        }`}
      >
        <div className="px-4 py-3">{children}</div>
      </div>
    </div>
  );
}

export default Accordion;

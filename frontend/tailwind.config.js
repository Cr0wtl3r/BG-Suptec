/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,js,ts}"],
  theme: {
    extend: {
      colors: {
        'dark-blue-bg': '#1A202C',
        'dark-blue-light': '#2D3748',
        'accent-orange': '#ed5f0c',
        'text-light': '#EDF2F7',
        'primary-dark': '#201f1f70',
        'primary-purple-dark': '#171958af',

      },
    },
  },
  plugins: [],
}
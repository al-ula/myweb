/** @type {import('tailwindcss').Config} */
import daisyui from "daisyui";
import catppuccin from "@catppuccin/daisyui";

export default {
  content: ["./templates/**/*.{html,hbs}", "./src/**/*.{rs}"],
  theme: {
    extend: {
      fontFamily: {
        hasklig: ["Hasklig"],
      },
    },
  },
  plugins: [daisyui],
  daisyui: {
    themes: ["light", "dark", catppuccin("latte"), catppuccin("mocha")],
  },
}


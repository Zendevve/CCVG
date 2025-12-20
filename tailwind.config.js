/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,js}"],
  theme: {
    extend: {
      colors: {
        // Tahoe semantic colors
        'glass': {
          'clear': 'rgba(255, 255, 255, 0.08)',
          'regular': 'rgba(255, 255, 255, 0.15)',
          'thick': 'rgba(255, 255, 255, 0.25)',
          'ultra': 'rgba(40, 40, 44, 0.85)',
        },
        'label': {
          'primary': '#FFFFFF',
          'secondary': 'rgba(235, 235, 245, 0.60)',
          'tertiary': 'rgba(235, 235, 245, 0.30)',
          'quaternary': 'rgba(235, 235, 245, 0.18)',
        },
        'tint': {
          'blue': '#0A84FF',
          'green': '#30D158',
          'red': '#FF453A',
          'orange': '#FF9F0A',
          'purple': '#BF5AF2',
          'indigo': '#5E5CE6',
          'teal': '#64D2FF',
        },
        'fill': {
          'primary': 'rgba(120, 120, 128, 0.36)',
          'secondary': 'rgba(120, 120, 128, 0.32)',
          'tertiary': 'rgba(120, 120, 128, 0.24)',
        },
      },
      borderRadius: {
        'tahoe-window': '26px',
        'tahoe-card': '20px',
        'tahoe-button': '14px',
        'tahoe-sm': '10px',
      },
      backdropBlur: {
        'glass': '30px',
        'glass-heavy': '50px',
      },
      backdropSaturate: {
        '180': '1.8',
      },
      fontFamily: {
        'system': ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Display', 'Segoe UI', 'Roboto', 'sans-serif'],
      },
      animation: {
        'spin-slow': 'spin 1.5s linear infinite',
        'pulse-soft': 'pulse 2s ease-in-out infinite',
      },
    },
  },
  plugins: [],
}

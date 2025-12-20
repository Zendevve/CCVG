/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,js}"],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // macOS 26 Tahoe Colors from reference
        'tahoe': {
          // Window backgrounds
          'window': 'rgba(40, 40, 42, 0.78)',
          'titlebar': 'rgba(50, 50, 52, 0.92)',
          'sidebar': 'rgba(35, 35, 37, 0.85)',
          'content': 'rgba(30, 30, 32, 0.95)',

          // Glass panels
          'glass': 'rgba(60, 60, 65, 0.55)',
          'glass-light': 'rgba(80, 80, 85, 0.40)',

          // Fills
          'fill': 'rgba(255, 255, 255, 0.08)',
          'fill-secondary': 'rgba(255, 255, 255, 0.05)',
        },
        // Labels
        'label': {
          'primary': '#FFFFFF',
          'secondary': 'rgba(255, 255, 255, 0.55)',
          'tertiary': 'rgba(255, 255, 255, 0.25)',
        },
        // System accents
        'accent': {
          'blue': '#007AFF',
          'green': '#30D158',
          'red': '#FF453A',
          'orange': '#FF9F0A',
          'purple': '#BF5AF2',
        },
        // Borders
        'border': {
          'glass': 'rgba(255, 255, 255, 0.18)',
          'separator': 'rgba(255, 255, 255, 0.08)',
        }
      },
      borderRadius: {
        'tahoe': '12px',
        'panel': '10px',
        'button': '8px',
        'field': '6px',
      },
      backdropBlur: {
        'window': '80px',
        'panel': '40px',
        'menu': '25px',
      },
      backdropSaturate: {
        '180': '1.8',
        '190': '1.9',
      },
      fontFamily: {
        'sf': ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'system-ui', 'sans-serif'],
      },
      fontSize: {
        '11': ['11px', '13px'],
        '13': ['13px', '16px'],
        '15': ['15px', '20px'],
        '17': ['17px', '22px'],
        '20': ['20px', '24px'],
        '22': ['22px', '28px'],
      },
    },
  },
  plugins: [],
}

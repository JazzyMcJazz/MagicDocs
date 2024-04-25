/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        './templates/**/*.html',
    ],
    theme: {
        extend: {
            borderWidth: {
                1: '1px',
            },
            colors: {
                base: '#414141',
            },
            gridTemplateColumns: {
                base: 'auto 1fr',
            },
        },
    },
    plugins: [
        require('@tailwindcss/forms'),
    ],
}

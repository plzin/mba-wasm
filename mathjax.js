// This module synchronously loads mathjax on import with custom settings.

// This is using an XMLHttpRequest to fetch the js and execute it with window.eval.
// Since synchronous XMLHttpRequests in the main thread are deprecated, this is a bit hacky.
// We use a asynchronous request and in the listener we resolve a promise which we wait for.
await new Promise((resolve) => {
    window.MathJax = {
        startup: {
            typeset: false,
        },
        options: {
            enableMenu: false,
        },
        chtml: {
            fontURL: 'https://cdn.jsdelivr.net/npm/mathjax@3/es5/output/chtml/fonts/woff-v2',
        },
    }

    const req = new XMLHttpRequest()
    req.open('GET', 'https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js')
    req.addEventListener('load', () => {
        // This stops styles for the (disabled) context menu from being loaded.
        // const code = req.responseText.replace(',y.CssStyles.addInfoStyles(this.document.document),y.CssStyles.addMenuStyles(this.document.document)', '')
        const code = req.responseText
        // Run the java script.
        window.eval(code)

        // Associate a function with MathJax that resets all state.
        window.MathJax.reset = () => {
            window.MathJax.typesetClear()
            window.MathJax.texReset()
            window.MathJax.startup.output.clearCache()
        }

        // Associate a function with MathJax that sets the css.
        window.MathJax.set_css = (id) => {
            let sheet = document.getElementById(id)
            if (sheet)
                sheet.remove()
            let css = window.MathJax.chtmlStylesheet()
            css.id = id
            // Remove some annoying css that gives me warnings.
            // css.innerText = css.innerText.replace(
            // '\n_::-webkit-full-page-media, _:future, :root mjx-container {\n  will-change: opacity;\n}',
            // '')
            document.head.appendChild(css)
        }
        resolve()
    })
    req.send()
})
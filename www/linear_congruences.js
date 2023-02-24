import './mathjax.js'
import { solve_congruences, Width } from './wasm.js';

// Global tex render options.
let tex_options = { scale: 1.2 }

// Get a bunch of DOM elements we need.
const mat = document.getElementById('matrix')
const mod = document.getElementById('modulus')
const b = document.getElementById('solve-btn')
const error = document.getElementById('err')
const system = document.getElementById('system')
const math = document.getElementById('math')

// Changing the matrix text field updates the tex'd system.
const show_eqs = () => {
    MathJax.reset()
    let eqs = '\\begin{align}'
    for (let line of mat.value.split('\n')) {
        line = line.trim()
        if (line === '')
            continue;

        let elements = line.split(' ')
        let rhs = ''
        if (elements.length > 1) {
            rhs = elements.pop()
        }
        for (let i = 0; i < elements.length; ++i) {
            eqs += elements[i]
            eqs += `x_{${i+1}}&+`
        }

        eqs = eqs.slice(0, -1)
        eqs += `\\equiv ${rhs}&\\pmod{2^{${mod.options[mod.selectedIndex].label}}}\\\\`
    }
    eqs += '\\end{align}'

    system.replaceChildren()
    system.appendChild(MathJax.tex2chtml(eqs, tex_options))

    MathJax.set_css('mjx-system-styles')
}
    
// Draw the equations once.
show_eqs()

// Update when the matrix or the modulus changes.
mat.oninput = mod.oninput = show_eqs

// Clicking the button solves the system.
b.onclick = () => {
    try {
        error.textContent = ''

        let add_text = (str) => {
            math.appendChild(document.createTextNode(str))
        }
        let add_math = (str) => {
            math.appendChild(MathJax.tex2chtml(str, tex_options))
        }

        let s = solve_congruences(mat.value, Width[mod.value])

        MathJax.reset()
        math.replaceChildren()
        add_text('The diagonalization of the matrix A is')
        add_math(s.diag)

        add_text('The new system is')
        add_math(s.scalar_system)

        add_text('which results in the single variable linear congruences')
        add_math(s.linear_solutions)

        if (s.vector_solution.length === 0) {
            add_text('So overall the system has no solution.')
        } else {
            add_text('The vector form is')
            add_math(s.vector_solution)

            add_text('The final solution to the original system is')
            add_math(s.final_solution)
        }

        MathJax.set_css('mjx-solution-styles')
    } catch (err) {
        if (typeof err === "string") {
            error.textContent = err
        } else {
            console.log(err)
            error.textContent = 'Error. Check console.'
        }
    }
}
import './mathjax.js'
import { Width, invert_poly, rand_poly } from './wasm.js'

const input = document.getElementById('input')
const input_error = document.getElementById('input-error')
const invert_btn = document.getElementById('invert-btn')
const rand = document.getElementById('rand-poly')
const algorithm = document.getElementById('algorithm')
const algorithms = document.getElementsByName('algorithm')
const output = document.getElementById('output')

// Setup handling for the algorithm dropdown.
for (const li of algorithms) {
    li.onclick = (e) => {
        for (const li of algorithms) {
            li.classList.remove('active')
        }
        e.target.classList.add('active')
        algorithm.textContent = e.target.textContent
    }
}

rand.onclick = () => {
    const bits = Width[document.querySelector('input[name=width]:checked').value]
    const p = rand_poly(bits)
    input.value = p
}

invert_btn.onclick = () => {
    const poly = input.value
    const bits = Width[document.querySelector('input[name=width]:checked').value]
    const alg = algorithm.innerText.trim()
    try {
        input.classList.remove('is-invalid')
        input_error.textContent = ''
        const inverse = invert_poly(poly, bits, alg)
        MathJax.reset()
        output.replaceChildren()
        output.appendChild(MathJax.tex2chtml(inverse, { scale: 1.3 }))
        MathJax.set_css('mathjax-styles')
    } catch (err) {
        output.replaceChildren()
        if (typeof err === 'string') {
            input.classList.add('is-invalid')
            input_error.textContent = err
        } else {
            console.log(err);
            input.classList.add('is-invalid')
            input_error.textContent = 'Check console.'
        }
    }
}
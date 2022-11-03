import { obfuscate, normalize_op, ObfuscateReq, Bitness, Printer } from './wasm.js';
import './mathjax.js'

const btn = document.getElementById('obfuscate-btn')
const input = document.getElementById('input')
const output = document.getElementById('output')
const input_error = document.getElementById('input-error')
const ops = document.getElementById('ops')
const op_input = document.getElementById('op-input')
const op_add = document.getElementById('op-add')
const op_add_item = document.getElementById('op-add-item')
const randomize = document.getElementById('random')
const output_type = document.getElementById('output-type')
const output_types = document.getElementsByName('output-type')

// Highlights inline code.
function hi_in(code) {
    return `<code class="language-c">${Prism.highlight(code, Prism.languages.c, 'c')}</code>`
}

// Part of the popover config supplying default values.
const popover_config = {
    trigger: 'hover',
    delay: { show: 500, hide: 100 },
    html: true,
    customClass: 'custom-popover',
}

// Popover for the input box.
new bootstrap.Popover(input, {
    ...popover_config,
    title: 'Expression that will be obfuscated',
    content:
`
Currently, the expression has to be a linear combination of boolean expressions.
Check 'What operations are allowed?' at the bottom of the page.
<br>
E.g. ${hi_in('3*(x & ~y) + 4*(x | y) - 2*~x')}.
<br>
This limitation will be removed in the future and more general expression will be allowed.
More commonly, you will probably want to use things like ${hi_in('x + y')}, ${hi_in('x - y')}.
<br>
Constants (${hi_in('1312')}) are also allowed.
`
})

// Popover for the rewrite operation input.
new bootstrap.Popover(op_input, {
    ...popover_config,
    title: 'Operations used during rewriting',
    content:
`
These are the operations that are allowed to appear in the resulting linear combination.
Only linear combinations of boolean operations are allowed.
`
})

// Popover for the 'random solution' button.
new bootstrap.Popover(document.getElementById('randomize-div'), {
    ...popover_config,
    title: `Generate a random output`,
    content:
`
Rewriting the input using the operations involves solving a 'System of Linear Congruences',
which is very similar to 'Systems of Linear Equations' that are known from Linear Algebra.
In the same way the solution also is a particular solution plus any vector in the kernel. 
If randomize solution is disabled, the particular solution that the algorithm returns is used.
Since the algorithm is deterministic it will always be the same.
Note that changing the order of the rewrite operations can change the solution.
To get a canonical solution, we could define some sort of norm and try to find the smallest
solution according to that norm, but this is future work.
`
})

// 'What is Mixed Boolean-Arithmetic?'
document.getElementById('acc-col-1').children[0].innerHTML =
`
Mixed Boolean-Arithmetic (MBA) is a name for expressions which contain both the usual arithmetic operations
(${hi_in('+')}, ${hi_in('-')}, ${hi_in('*')}, ${hi_in('/')})
as well as boolean operations (${hi_in('&')}, ${hi_in('|')}, ${hi_in('^')}, ${hi_in('~')}).
A simple example is the expression ${hi_in('2 * (x & y) + (x ^ y)')} that computes ${hi_in('x + y')}.
This particular one works for integers of any number of bits but generally they can be specific to
a certain size, such as the following one, that also computes ${hi_in('x + y')} but only for 8-bit integers:
<pre class="language-c" style="font-size: .875rem">${Prism.highlight('-38*(x & y) - 83*(x ^ y) - 64*~(x ^ (y ^ z))\n - 41*~x - 43*~y - 23*y - 44*z - 20*(y & z)\n - 21*(x | z) - 107*(~x & z) - 108*(y | ~z)', Prism.languages.c, 'c')}</pre>
`

// 'What operations are allowed?'
document.getElementById('acc-col-2').children[0].innerHTML =
`
Currently, this site can only generate <strong>Linear</strong> MBA expressions, which are linear combinations of boolean operations,
such as ${hi_in('24*(x & y) - 22*(x | y) - 105*(x ^ y) + 128*~x + 128*~y')}.
The allowed boolean operations are ${hi_in('x & y')} (and), ${hi_in('x | y')} (or), ${hi_in('x ^ y')} (xor), ${hi_in('~x')} (equivalently ${hi_in('!x')}) (not) and ${hi_in('-1')}.
${hi_in('-1')} is the mnemonic for the constant 1 function on each bit, which (when interpreted in two's complement) has the value -1.
Note that ${hi_in('!')} is an alias for the logical NOT and the same as ${hi_in('~')} here, whereas this is not the case in C.
Of course the operations can also be nested: ${hi_in('x & (y | !z)')}.
Constants ${hi_in('1312')} are also allowed as part of the linear combination and are represented internally as ${hi_in('-1312*(-1)')}.
<br><br>
The rewrite operations are usually just the boolean operations that appear in the output linear combination,
but they can be linear combinations themselves, as using those is equivalent to restricting the coefficients
of the output.
<br><br>
Additionally, the input expression has to be a Linear MBA expression as well, but this will be relaxed in the future.
The idea is to obfuscate parts of a general expression that are Linear MBA and substitute those in, so often it will
just be something like ${hi_in('x+y')}.
The rewrite operations are usually the boolean operations that appear in the linear combination,
but can also be linear combinations of boolean operations as using those is equivalent to restricting
the coefficients in the output linear combination.
`

// Setup handling for the output type dropdown.
for (const li of output_types) {
    li.onclick = (e) => {
        for (const li of output_types) {
            li.classList.remove('active')
        }
        e.target.classList.add('active')
        output_type.textContent = e.target.textContent
    }
}

// Remove an operation from the op list.
const remove_op = (e) => {
    e.target.parentNode.remove()
}

// Add an operation to the list of operations used for rewriting.
const add_op = () => {
    // Normalize the operation and make sure it is valid.
    const bits = Bitness[document.querySelector('input[name=bitness]:checked').value]
    const s = normalize_op(op_input.value, bits)
    
    // If it isn't, indicate that.
    if (s === '') {
        op_input.classList.add('is-invalid')
        op_input.parentElement.classList.add('is-invalid')
        return
    }

    // Remove potential prior indication.
    op_input.classList.remove('is-invalid')
    op_input.parentElement.classList.remove('is-invalid')

    // Create a new list item.
    const list_item = document.createElement('li')
    list_item.classList.add('list-group-item', 'd-flex', 'justify-content-between', 'align-items-start')

    const op_text = document.createElement('div')
    op_text.classList.add('ms-2', 'me-auto')
    op_text.textContent = s
    op_text.setAttribute('name', 'op-value')
    list_item.appendChild(op_text)

    const remove_btn = document.createElement('button')
    remove_btn.classList.add('btn-close')
    remove_btn.type = 'button'
    remove_btn.onclick = remove_op
    list_item.appendChild(remove_btn)

    // Insert it before the last item, which is the one
    // that contains the textfield and button.
    ops.insertBefore(list_item, op_add_item)
}

op_add.onclick = add_op
op_input.onkeydown = (e) => {
    if (e.key == "Enter")
        add_op()
}

// Do the obfuscation.
btn.onclick = () => {
    let req = new ObfuscateReq()
    req.expr = input.value
    req.randomize = randomize.checked

    const printer = Printer[output_type.innerText.trim()]
    req.printer = printer

    // Collect the rewrite ops.
    for (const e of document.getElementsByName('op-value')) {
        req.add_op(e.innerText)
    }

    // Get the number of bits we are obfuscating for.
    const bits = Bitness[document.querySelector('input[name=bitness]:checked').value]
    req.bits = bits

    try {
        // Do the rewriting.
        const s = obfuscate(req)
        input.classList.remove('is-invalid')
        input_error.textContent = ''

        // Display the result.
        output.replaceChildren()
        if (printer == Printer.C) {
            const code = document.createElement('pre')
            code.classList.add('language-c')
            code.innerHTML = Prism.highlight(s, Prism.languages.c, 'c')
            output.appendChild(code)

            // Very hacky and requires the code to contain commas only for the arguments.
            const args = s.split(',').map(() => '0').join(', ')
            const ce_code = encodeURIComponent(`#include <cstdint>\n#include <iostream>\n\n${s}\n\nint main() {\n\tstd::cout << ${bits == Bitness.U8 ? '(uint32_t)' : ''}f(${args}) << "\\n";\n}`)
            const ce_btn = document.createElement('button')
            ce_btn.textContent = 'Open in Compiler Explorer'
            ce_btn.classList.add('btn', 'btn-secondary')
            ce_btn.onclick = () => {
                window.open(`https://godbolt.org/#g:!((g:!((g:!((h:codeEditor,i:(filename:'1',fontScale:14,fontUsePx:'0',j:1,lang:c%2B%2B,selection:(endColumn:1,endLineNumber:1,positionColumn:1,positionLineNumber:1,selectionStartColumn:1,selectionStartLineNumber:1,startColumn:1,startLineNumber:1),source:'${ce_code}'),l:'5',n:'0',o:'C%2B%2B+source+%231',t:'0')),k:50,l:'4',n:'0',o:'',s:0,t:'0'),(g:!((h:executor,i:(argsPanelShown:'1',compilationPanelShown:'0',compiler:clang_trunk,compilerOutShown:'0',execArgs:'',execStdin:'',fontScale:14,fontUsePx:'0',j:1,lang:c%2B%2B,libs:!(),options:'',source:1,stdinPanelShown:'1',tree:'1',wrap:'1'),l:'5',n:'0',o:'Executor+x86-64+clang+(trunk)+(C%2B%2B,+Editor+%231)',t:'0')),k:50,l:'4',n:'0',o:'',s:0,t:'0')),l:'2',n:'0',o:'',t:'0')),version:4`)
            }
            output.appendChild(ce_btn)
        } else if (printer == Printer.Rust) {
            const code = document.createElement('pre')
            code.classList.add('language-rust')
            code.innerHTML = Prism.highlight(s, Prism.languages.rust, 'rust')
            output.appendChild(code)

            const pg_btn = document.createElement('button')
            pg_btn.textContent = 'Open in Rust Playground'
            pg_btn.classList.add('btn', 'btn-secondary')

            // Very hacky and requires the code to contain commas only for the arguments.
            const args = s.split(',').map(() => 'Wrapping(0)').join(', ')
            const pg_code = encodeURIComponent(`use std::num::Wrapping;\n\nfn main() {\n\tprintln!("{}", f(${args}));\n}\n\n${s}`)
            pg_btn.onclick = () => {
                window.open(`https://play.rust-lang.org/?version=stable&mode=release&edition=2021&code=${pg_code}`)
            }
            output.appendChild(pg_btn)
        } else if (printer == Printer.Tex) {
            MathJax.typesetClear()
            MathJax.texReset()
            MathJax.startup.output.clearCache()
            output.appendChild(MathJax.tex2chtml(s, { scale: 1.3 }))

            let sheet = document.getElementById('mathjax-styles')
            if (sheet)
                sheet.remove()
            sheet = MathJax.chtmlStylesheet()
            sheet.id = 'mathjax-styles'
            document.head.appendChild(sheet)
        } else {
            output.textContent = s
        }
    } catch (err) {
        input.classList.add('is-invalid')
        output.textContent = ''
        if (typeof err === 'string') {
            input_error.textContent = err
        } else {
            console.log(err)
            input_error.textContent = 'Unknown error. Check console.'
        }
    }
}
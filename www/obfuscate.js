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
    const s = normalize_op(op_input.value)
    
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
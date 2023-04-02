import { obfuscate, Width, Printer } from './wasm.js';

const btn = document.getElementById('obfuscate-btn')
const input = document.getElementById('input')
const output = document.getElementById('output')
const input_error = document.getElementById('input-error')
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

// Do the obfuscation.
btn.onclick = () => {
    const expr = input.value

    const printer = Printer[output_type.innerText.trim()]

    // Get the number of bits we are obfuscating for.
    const bits = Width[document.querySelector('input[name=bitness]:checked').value]

    try {
        // Do the rewriting.
        const s = postprocess_code(obfuscate(expr, bits, printer))
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
            const ce_code = encodeURIComponent(`#include <cstdint>\n#include <iostream>\n\n${s}\n\nint main() {\n\tstd::cout << ${bits == Width.U8 ? '(uint32_t)' : ''}f(${args}) << "\\n";\n}`)
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
        }
        else {
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

// Hide this ugly code down here.
function postprocess_code(code) {
    let s = ''
    let lines = code.split('\n')
    //s += lines[0]
    //s += '\n'
    //lines = lines.slice(1)
    for (let l of lines) {
        let tabs = 0
        for (; tabs < l.length && l[tabs] == '\t'; tabs++) {}
        l = l.substring(tabs)

        inner: for (let j = 0; ; j++) {
            let max = 68 - 4 * tabs;
            if (j == 0) {
                s += '\t'.repeat(tabs)
            } else {
                s += '\t'.repeat(tabs+1)
                max -= 4;
            }

            if (l.length >= max) {
                for (let i = max; i >= 0; i--) {
                    if (l[i] == ' ') {
                        s += l.substring(0, i)
                        s += '\n'
                        l = l.substring(i+1)
                        continue inner
                    }
                }
            }

            // If we never found a space then just put the whole string into the line anyways.
            s += l
            s += '\n'
            break
        }
    }

    return s
}
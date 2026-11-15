# PhotoCraft no navegador do celular

A branch `web/mobile-browser` adapta a versão web existente para celulares. Ela
reutiliza os controles de toque da versão Android, mas roda em Rust/WebAssembly
no navegador, sem instalar APK e sem enviar as fotos a um servidor.

## Interface

- Telas de até 720 pixels CSS usam controles compactos automaticamente.
- Celulares e tablets com ponteiro principal de toque também usam essa interface
  em paisagem (até 1400 pixels CSS no maior lado).
- As barras de ações e ferramentas podem ser deslizadas horizontalmente.
- Camadas ficam em um painel inferior; Opções mostra os ajustes da ferramenta.
- O tamanho acompanha a área visível do navegador, inclusive quando o teclado
  abre; as margens de segurança reservam espaço para recortes e indicadores.
- `?mobile` força os controles de toque; `?desktop` força a interface completa.
  Combine com `?webgl&mobile` para forçar também o renderizador WebGL2.
- No Safari e nos navegadores do iOS (WebKit), o site baixa o WebAssembly inteiro
  antes de compilá-lo, em vez de compilar durante o download. `?stream` força a
  compilação durante o download e `?buffered` força o download inteiro.
- Enquanto carrega, a página mostra a etapa (download, início) e os erros
  diretamente na tela, já que o console não fica acessível no celular.

## Compilar e abrir

Instale Rust estável e Trunk. Na raiz do repositório:

```sh
rustup target add wasm32-unknown-unknown
cd apps/photocraft-web
trunk serve --release --address 0.0.0.0 --port 8765
```

Abra o endereço de rede do computador no celular, na porta 8765. HTTP na rede
local usa WebGL2; para WebGPU e recursos da área de transferência, sirva por
HTTPS. O navegador precisa oferecer WebGPU ou WebGL2.

Para gerar arquivos estáticos prontos para hospedagem:

```sh
packaging/web/package.sh
```

O ZIP gerado em `dist/` contém o site e as licenças. Extraia-o e publique o
conteúdo da pasta interna em uma hospedagem estática HTTPS. Não precisa de
backend. Os caminhos relativos permitem hospedar sob um prefixo. Consulte
[as instruções de hospedagem](../packaging/web/README.md).

## Desempenho

O build web usa WebAssembly SIMD (`.cargo/config.toml`) e compila filtros, ajustes,
compositor e gerenciamento de cor otimizando velocidade (`opt-level = 3` no perfil
`wasm-release`). Isso exige Safari 16.4 / iOS 16.4 ou mais novo, o mesmo mínimo
que o carregamento em partes já pedia, e deixa o download um pouco maior.

## Arquivos e limites

Abrir usa o seletor de arquivos do navegador. Salvar e exportar geram downloads;
confira o arquivo baixado antes de fechar a aba. Preferências usam localStorage
e os presets de pincel usam IndexedDB, conforme permitido pelo navegador.
O projeto ainda não oferece recuperação automática dos documentos na web.

Esta adaptação conserva o editor original e seus limites de memória. Comece com
imagens pequenas em celulares. Algumas janelas e controles avançados ainda têm
dimensões de desktop; a interface compacta prioriza abrir, desenhar, recortar,
camadas e exportar. A validação em aparelhos reais, especialmente teclado e
seletor de arquivos do Safari, continua necessária.

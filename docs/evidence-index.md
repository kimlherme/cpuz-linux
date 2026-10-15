# Índice de evidências

Este índice relaciona conclusões aos registros REA das duas passagens estáticas. Os registros completos permanecem no acervo privado da investigação; não são redistribuídos junto aos executáveis proprietários neste repositório.

## Identidade e interpretação

- Alvo de código: CPU-Z 3.01 x64, SHA-256 `a01369043c9e199747d62b950fb6db3f793ea74feb9c667ae291988405905fbe`.
- Provider: Ghidra 12.1.2 por REA.
- Passagens: localização da carga e rastreio da pontuação single.
- Fonte deste índice: projeções `focused-evidence.json` e `score-single-focused-evidence.json`, com IDs preservados.
- Operações `procedure_pseudo_code` produzem interpretação do decompilador; não recuperam código-fonte original.
- Operações `procedure_assembly` e `read_bytes` sustentam a revisão de retorno, offsets e formatos.
- Todos os endereços são VA do x64 identificado, não offsets de arquivo nem endereços de outras versões.

## Registros selecionados

| Evidence ID | Operação | Parâmetro/endereço | Conclusão apoiada | Limite de cobertura |
| --- | --- | --- | --- | --- |
| `ev_1631e2df47b49dc7e183d5c9af7b34c9ca9f0785c98dd8035b0c7a61aafb4d7c` | `binary_overview` | `Inventário` | Identidade e contexto do PE x64 | Inventário estático; PDB referenciado não carregado |
| `ev_2d82aef7dd47c1695a31f4b551d810a7061cab42714a44b69909d6141540e503` | `search_strings` | `17.01` | String 17.01 | String isolada não comprova seleção em execução |
| `ev_c343d61d8937b8abc1fdbb2456ee399cd94599ecc51d0f578445b359cdbb7c29` | `search_strings` | `19.01` | Strings 19.01 | Não estabelece disponibilidade por CPU |
| `ev_51dfc626a455ae96bd8eaf535418df4854321dc54502cfcaa8cff39399ed726d` | `procedure_pseudo_code` | `0x140349dee` | Criação dos grupos de resultados | Complementa os nomes encontrados |
| `ev_271df63cd7df9af52ebe52e05024aa1d953637d2410ca0c0c925dacf5500471a` | `procedure_pseudo_code` | `0x140349fd2` | Seleção CNoise2 / CNoise3 | Associação estática de identificadores |
| `ev_bf2754e5547b271586d567ae207ac91d5d2bf7046e3983e41b0479a8be232839` | `read_bytes` | `0x1404387f8` | Vtable CNoise2 | Bytes da tabela no alvo identificado |
| `ev_47a4c149fdd289284964104c9059b66bbdea6b64b7a8fc894a79bb8a8d22337e` | `read_bytes` | `0x140436da0` | Vtable CNoise3 | Não reconstrói o algoritmo 19.01 |
| `ev_fcbb62a38d7824d50de05b85232095f5744de3f7736872bfafd4bdbbe381134f` | `procedure_pseudo_code` | `0x1400052e0` | Inicialização e tabela de 4096 floats | Auxiliar numérica não completamente especificada |
| `ev_475280242223b774b6808a16128b19d0eacf839ecf99b8a0878c6a9d852fd2c8` | `procedure_pseudo_code` | `0x1400053b0` | Gerador de gradientes e consultas à tabela | Sem vetores de execução do original |
| `ev_5bcb6eaac5318a24b940a1598a866de220e2a3dcdb74b6290dd78ce9bd80864c` | `procedure_pseudo_code` | `0x140005550` | Estrutura simplex 2D | Identificação algorítmica inferida |
| `ev_306a89f56159bcf782f5c9c1cede3fb00adaa9d65bd1fc873b7924440da90a47` | `procedure_assembly` | `0x140005550` | Operações escalares SSE no núcleo | Não é inventário global do executável |
| `ev_16993a63964b4776e328c9f7cb3122f1d525983b50d10801c36679cbc81ad791` | `procedure_pseudo_code` | `0x1400058a0` | Campo de 10 + 10 avaliações | Contratos numéricos precisam de validação |
| `ev_6da4b8051f32b41d4f35e7ebc84e9cb8f34036417464f7ab14c923ecba50e947` | `procedure_pseudo_code` | `0x140005aa0` | Campo de 7 + 7 avaliações | Mesma limitação numérica |
| `ev_c5e8be74898f100dd287b1f9b94bd5bd9c9d8dea8cc5f67ae80bb6cf78a15fd6` | `procedure_pseudo_code` | `0x140005cb0` | Composição e quatro bytes por pixel | Interpretação visual dos canais não observada |
| `ev_407802acf83347b6b74fb20db3b319940666791748e9aff320e97763444587fc` | `procedure_pseudo_code` | `0x140005f30` | Worker reporta comprimento de trabalho | Não mede ciclos ou instruções |
| `ev_74db3f797133b5aae68c10afe10d98ff4b02a55f2753acfa0aeb49fc8c95b736` | `procedure_pseudo_code` | `0x140349ca0` | Grade 1024 × 1024 e 4 MiB | Alocação observada estaticamente |
| `ev_9d03eed9040cf6a6d188779d262288da85686ac82260d903fe0b520d7623755f` | `procedure_pseudo_code` | `0x140349910` | Duração 10000 e sequência de fases | Assinaturas do decompilador exigem assembly |
| `ev_21f5654166e5dc70fa1b2cd6817c85d61fca5de91d09da5b6af237b619d2cbc0` | `procedure_pseudo_code` | `0x140318a10` | Scheduling multi Windows | APC, eventos e afinidade não executados |
| `ev_1c5200968e112c63031f66c9c644344c5c653393bb003c77cbe8458bce5961cb` | `procedure_pseudo_code` | `0x140318e40` | Scheduling single Windows | Retorno void inferido incorretamente |
| `ev_d3bec3f650e6dde86970a02fcea7507aa7029ab24863657e42d95131866b0b19` | `procedure_pseudo_code` | `0x140349b50` | Criação e prioridade de thread | Política Linux é uma adaptação |
| `ev_223b09d44fdafeee5ee6f314e70b2edfc42bb0d59c30c9bb099ea2a9ffbaf5bc` | `procedure_assembly` | `0x140318a10` | Retorno numérico multi em XMM0 | Assembly complementa pseudocódigo |
| `ev_86ea33c0b2bc9251500b27712e3be149848d5cdc40ee9688423e8e8b1ba925cb` | `procedure_assembly` | `0x140318e40` | Pixels / tempo inteiro ms no single | Não houve execução do original |
| `ev_f41adf6cd68069d32c76f6637a75fdc85bd46fb364fd671a66ca93d735c4c19e` | `procedure_pseudo_code` | `0x1400cfbb4` | Implementação de timer chama QPC | Seleção concreta por máquina não demonstrada |
| `ev_1cb5810b79632a20698b1e02de8fb2ebf38485be99c960b898c36c7c5ef156ee` | `procedure_assembly` | `0x140349910` | Slots e campos single/multi | Cópias sem normalização nos trechos seguidos |
| `ev_e6dca12f3c78e2418605fecb1eb88d20bed5d96578f99be86487c0ec5ea92046` | `procedure_pseudo_code` | `0x140347d20` | UI absoluta %.1f e percentuais separados | Conclusão restrita ao caminho absoluto |
| `ev_62cc80873c712bf60c96f9db41453e2820976899b77e2d2f6230402538e0a9ad` | `procedure_pseudo_code` | `0x140320260` | Contexto do export estruturado | Pseudocódigo omite argumentos float |
| `ev_b682e236b89bb65e56ba96432a1f6a5a0d2d0fd9a9ef909c0e1361a688d75b2f` | `read_bytes` | `0x140320250` | Bytes das cargas +208/+20c e chamadas | Resolve ambiguidade do pseudocódigo |
| `ev_0a4888323b69188b9878f94af08ddd338cce581f3240dec4404609716bc3afef` | `read_bytes` | `0x14041f930` | Formatos %d e chaves st/mt | Não generaliza para todos os exports |
| `ev_b4ebe45ab8db5c48c57ac54f9ad86523498bfec2ea78aa2a534e47d3b7660c35` | `procedure_pseudo_code` | `0x1400363cc` | Arredondamento inteiro positivo | Sem escala adicional nesse auxiliar |

## Rastreio do resultado single

| Etapa | Localização no alvo | Evidência principal |
| --- | --- | --- |
| Unidade reportada | Worker `0x140005f30`, campo `+0x3c` | `ev_407802acf83347b6b74fb20db3b319940666791748e9aff320e97763444587fc` |
| Soma ao contador | `0x140319045` | `ev_86ea33c0b2bc9251500b27712e3be149848d5cdc40ee9688423e8e8b1ba925cb` |
| Leitura final | `0x140319179` | Mesmo registro de assembly single |
| Ticks × 1000 | `0x140319199` | Mesmo registro |
| Divisão inteira pela frequência | `0x1403191a2` | Mesmo registro |
| Conversão contador/tempo | `0x1403191a6`, `0x1403191a9` | Mesmo registro |
| Divisão e retorno XMM0 | `0x1403191bc` | Mesmo registro |
| Armazenamento single bruto | `0x140349a6b`, campo `+0x21c` | `ev_1cb5810b79632a20698b1e02de8fb2ebf38485be99c960b898c36c7c5ef156ee` |
| Cópia para export | `+0x21c → +0x208` | Mesmo registro do controlador |
| Formatação absoluta | `0x140347ea3–0x140347ecc` | `ev_e6dca12f3c78e2418605fecb1eb88d20bed5d96578f99be86487c0ec5ea92046` |
| Export inteiro | `0x140320260`, auxiliar `0x1400363cc` | Registros de export, bytes e arredondamento |

## Conclusões que os registros não sustentam

| Afirmação | Por que permanece aberta |
| --- | --- |
| O CPU-Z foi executado pela investigação | As duas passagens foram estáticas |
| O timer QPC é selecionado em toda máquina | Foi localizada uma implementação, sem observar seleção em execução |
| O kernel Linux gera exatamente os mesmos pixels | Não foram disponibilizados vetores de referência do original |
| 19.01 usa a mesma carga e escala | As variantes foram identificadas, não reconstruídas integralmente |
| Todos os exports usam o mesmo arredondamento | Foi seguido um caminho estruturado específico |
| 523 e 535 são equivalentes dentro do erro | Há dois relatos do usuário, sem série controlada |
| O projeto é uma clean room isolada | Não existe registro de isolamento do implementador |
| As instruções Rust são idênticas às do original | Não houve comparação completa dos códigos gerados |

## Limitações do provider e da seleção

O Ghidra infere tipos, assinaturas e limites de funções. A conclusão de pontuação requereu assembly porque duas funções de medição foram descompiladas como void. O export também exigiu revisão de bytes porque argumentos float não apareciam corretamente no pseudocódigo.

A ausência de PDB completo limita nomes e tipos originais. Buscas por strings cobrem os resultados retornados e não estabelecem que todo comportamento do programa foi localizado. A presença de uma vtable ou API não equivale à observação de seu uso numa execução real.

As projeções consultadas não substituem os bundles completos e suas limitações. IDs preservados ajudam a auditoria por quem possui esse acervo; não concedem automaticamente acesso público aos registros. Para nova investigação, é necessário obter o mesmo alvo, conferir o hash e registrar um novo contexto de ferramentas.

## Documentos relacionados

- [Investigação e interpretação dos achados](reverse-engineering.md)
- [Metodologia e plano de validação](methodology.md)
- [Uso e instalação](../README.md)

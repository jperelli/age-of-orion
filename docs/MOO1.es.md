# Age of Orion y Master of Orion

Age of Orion se juega con las reglas de **Master of Orion** (SimTex, 1993). Las reglas están
portadas fórmula por fórmula de [1oom](https://github.com/1oom-fork/1oom), el motor de código
abierto que reproduce MOO1 v1.3, y comparadas con él turno por turno. Las pantallas, los
textos, las imágenes y la ambientación del tema Era son nuevos.

Hay dos temas con las mismas reglas:

- **Clásico**: la ambientación de MOO1, con estrellas, planetas, naves y razas alienígenas.
  Muestra las imágenes, la música, los sonidos y las fuentes de MOO1 si importas los archivos
  LBX de tu propia copia del juego.
- **Era**: el mismo juego como un mapa del mundo medieval. Los planetas son ciudades, las
  flotas son ejércitos y las diez razas son imperios medievales reales (tablas más abajo).

[English](MOO1.md)

## Qué es exactamente igual a Master of Orion

"Exactamente" quiere decir la misma fórmula y el mismo orden de operaciones que 1oom, con los
mismos números aleatorios.

- **Números aleatorios**: el generador de 1oom, así que las mismas reglas sacan las mismas
  tiradas.
- **Economía**: los cinco controles de gasto, incluido cómo se empujan entre sí y cómo uno
  bloqueado no se mueve. También la producción, las fábricas y su costo, la contaminación y su
  limpieza, el crecimiento de la población, la terraformación y el enriquecimiento del suelo,
  los impuestos y la reserva, y el mantenimiento de naves y bases.
- **Investigación**: el árbol de cada campo y la lista al azar de tecnologías que cada imperio
  puede investigar. También los costos, los intereses y los avances, y los niveles de campo,
  que abaratan y achican las piezas de las naves. El proyecto de un campo se elige cuando no
  tiene ninguno (al empezar y después de cada descubrimiento) y no se puede cambiar hasta
  descubrirlo.
- **Diseño de naves**: cada casco y cada pieza, con el costo, el espacio y los valores de
  combate de MOO1. Los cinco diseños iniciales cuestan 8, 14, 74, 65 y 570 BC, como en MOO1.
- **Movimiento**: las distancias del mapa, las velocidades, el alcance de combustible y los
  tanques de reserva, las nebulosas, los portales estelares y los transportes. Una flota en
  vuelo solo puede cambiar de destino con Comunicaciones hiperespaciales.
- **Combate**:
  - la cuadrícula táctica, la iniciativa, los rayos y misiles, cada especial de combate y la
    retirada
  - las batallas entre imperios de la computadora
  - el bombardeo orbital, los transportes derribados y el combate terrestre
- **Jugadores de la computadora**: la IA original (opción "Clásico") de producción,
  investigación, diseño de naves, flotas, transportes, diplomacia, espionaje y votación.
  También está la variante "Clásico+" de 1oom, con menos errores de la IA.
- **Diplomacia y eventos**:
  - el contacto, las relaciones, los tratados, el comercio, las audiencias, los espías y el
    sabotaje
  - los 16 eventos al azar, los dos monstruos espaciales y el Guardián de Orión
  - las rebeliones y los golpes de estado
  - el Consejo Galáctico y la guerra final contra el imperio que rechaza su veredicto
- **Errores**: las reglas de 1oom son las de MOO1 v1.3 con las correcciones oficiales de la
  v1.3a. Los errores de MOO1 que son parte de cómo se juega se mantienen;
  [MOO1_QUIRKS.md](MOO1_QUIRKS.md) los enumera (en inglés).

**Cómo se comprueba.** El arnés de comparación ([HARNESS.md](HARNESS.md), en inglés) toma las
partidas guardadas año por año de partidas de 1oom solo con la computadora. Juega cada año en
Age of Orion y compara cada planeta, imperio, flota y evento con la partida de 1oom del año
siguiente. En cinco partidas de 100 a 250 años, 613 de 710 turnos (86%) son idénticos, incluidos
los primeros 39 turnos de la partida pequeña.

## Qué es distinto

### Reglas

- **Generación de la galaxia**: la misma semilla todavía no da la galaxia de 1oom, porque la
  ubicación de los planetas natales está simplificada.
- **Turnos que todavía difieren**: el 14% de los turnos del arnés. Las diferencias están en
  algunos cambios de relaciones, batallas poco frecuentes entre imperios de la computadora,
  algunos resultados de bombardeos y algunos eventos.
- **Misiles en las batallas tácticas**: un misil alcanza una nave cuando llega al centro de su
  casilla. MOO1 usa un punto de cada imagen de nave, guardado en FIRING.LBX.
- **Flotas en un portal estelar**: no se pueden redirigir. En MOO1 esa flota conserva la
  velocidad del portal y puede llegar a cualquier estrella en un turno.
- **Grupos de tecnologías**: el espionaje y el intercambio de tecnologías usan grupos
  derivados de los nombres de las tecnologías. MOO1 guarda estos grupos en RESEARCH.LBX.
- **Jugadores**: un jugador humano por partida, como en MOO1. Las partidas de 1oom con varios
  jugadores en la misma computadora no están disponibles.
- **Partidas guardadas**: se guardan en archivos JSON. No se pueden cargar partidas de MOO1 ni
  de 1oom; el arnés puede convertir las de 1oom.

### Pantallas y presentación

- **Interfaz**: nueva, no las pantallas de 320x200 de MOO1, pero ordenada como ellas:
  - el mapa con un panel lateral, y los botones de MOO1 abajo (Juego, Diseño, Flota, Mapa,
    Razas, Planetas, Tecnología, Turno siguiente)
  - las flotas reciben órdenes como en MOO1: clic en las naves junto a una estrella, cuántas
    de cada una, clic en el destino y Aceptar
  - la pantalla de Tecnología tiene a la izquierda las tecnologías conocidas de cada campo y a
    la derecha los controles de investigación; el proyecto de una tecnología nueva se elige en
    su propio diálogo
  - los controles deslizantes de MOO1: − y + a cada lado, y un clic en el nombre lo bloquea
  - como en MOO1, de una estrella inexplorada solo se ve la distancia (ni nombre ni datos),
    las estrellas son puntos pequeños que titilan (las imágenes de estrellas de MOO1 si se
    importan), la pantalla de Razas muestra solo las razas conocidas (las que están fuera de
    contacto, en gris), y la imagen de una nave se elige con flechas en el diseño
  - a diferencia de MOO1, el mapa tiene zoom, hay atajos de teclado y los diálogos se adaptan
    a la ventana
- **Agregados**: la pantalla de Tecnología muestra exactamente qué da cada tecnología, con sus
  números, y un árbol tecnológico de lo ya descubierto. Un clic en la imagen de un planeta abre
  una ventana con su terreno, las naves en órbita y su población, fábricas, desechos y bases
  dibujados como filas de íconos. Un clic en el tipo, la estrella, el especial, el ambiente o
  el dueño de un planeta explica qué significan en las reglas (población, producción,
  crecimiento, habilidades de la raza). También puedes cambiar el idioma y el tema, y cargar
  mods.
- **Textos**: escritos para este juego, porque los de MOO1 tienen derechos de autor. En
  inglés, el tema Clásico usa las noticias y las descripciones de tecnologías de MOO1 si
  importas EVENTMSG.LBX y RESEARCH.LBX.
- **Imágenes**:
  - el Clásico muestra el arte de MOO1 que importes: planetas, razas, naves, fondos de
    batalla y las pantallas de desembarco y del noticiero. Sin él, el Clásico usa el arte
    propio del juego.
  - Era tiene su propio arte y retratos pintados.
- **Música y sonido**: las melodías y los efectos de sonido de MOO1 salen de tus MUSIC.LBX y
  SOUNDFX.LBX. Un sintetizador integrado toca las melodías, así que suenan distinto que en una
  placa AdLib o Roland. Sin esos archivos, el juego genera sus propios efectos de sonido.
- **Fuentes**: el tema Clásico usa las fuentes propias de MOO1, de FONTS.LBX.
- **Plataformas e idiomas**: funciona en el navegador y en la computadora, en inglés y en
  español.

## El tema Era

El tema Era cambia nombres, imágenes y textos, nunca reglas. Cada raza de MOO1 se convierte en
un imperio medieval cuya historia encaja con las habilidades de la raza, y su gobernante
pintado lleva un guiño a la raza.

### Imperios

<!-- generated:races -->
| Raza de MOO1 | Su habilidad | Imperio de Era | Gobernante | Por qué encaja | La raza en el retrato |
|---|---|---|---|---|---|
| **Humanos** | Investigación barata en Campos de fuerza, Planetología y Propulsión. | **Sacro Imperio Romano Germánico** | Emperador Federico Barbarroja | Los mejores diplomáticos y comerciantes: un imperio de príncipes, electores y ciudades libres de comerciantes. | Un diplomático con una carta sellada. |
| **Mrrshans** | +25% de daño con las armas, investigación de Armas barata. | **Imperio mongol** | Gengis Kan | Guerreros felinos con +25% de daño: los arqueros a caballo que conquistaron casi toda Eurasia. | Ojos ambarinos de gato y piel de leopardo de las nieves. |
| **Silicoides** | Puede colonizar cualquier planeta e ignora la contaminación, pero crece despacio. | **Imperio gúrida** | Sultán Muhammad de Ghor | Seres de roca que se asientan en cualquier lugar, no contaminan y crecen despacio: un imperio de las duras montañas afganas. | Piel gris como la piedra y un turbante con piedras sin tallar. |
| **Sakkras** | Crecimiento de población doble, Planetología barata. | **Imperio jemer** | Rey Suryavarman II | Reptiles cuya población crece el doble de rápido: los arrozales y embalses de Angkor alimentaron a un pueblo enorme. | Una corona de serpientes naga y oro con dibujo de escamas. |
| **Psilons** | +50% de investigación y todos los campos más baratos. | **Califato abasí** | Califa al-Mamún | Científicos con +50% de investigación: los califas de la Casa de la Sabiduría de Bagdad. | Un astrolabio en la Casa de la Sabiduría. |
| **Alkaris** | Las naves reciben 20% menos de daño, Propulsión barata. | **Imperio del Mar del Norte** | Rey Canuto el Grande | Pájaros cuyas naves son rápidas y difíciles de alcanzar: los drakares de Canuto dominaron el mar del Norte. | Un cuervo en el hombro y una capa de plumas de cuervo. |
| **Klackons** | Los trabajadores producen el doble, Construcción barata. | **Imperio Song** | Emperador Shenzong | Una colmena de insectos cuyos obreros producen el doble: los grandes talleres de hierro y porcelana de los Song. | Una armadura laqueada como el caparazón de un escarabajo y un sombrero con largas alas como antenas. |
| **Bulrathis** | +25 en el combate terrestre. | **Primer Imperio búlgaro** | Kan Krum | Osos, +25 en el combate terrestre: los guerreros búlgaros que vencieron a Bizancio en tierra. | Piel de oso y un casco de hierro. |
| **Meklars** | Cada colono opera 2 fábricas más, Computadoras baratas. | **Imperio bizantino** | Emperador Teófilo | Cíborgs cuyos colonos atienden más fábricas: Bizancio, famoso por sus autómatas y el fuego griego. | Un león mecánico de oro. |
| **Darloks** | +2 de alcance de los sensores. | **República de Venecia** | Dux Enrico Dandolo | Espías que cambian de forma: Venecia, cuyo Consejo de los Diez tuvo la mejor red de espías de su época. | Una máscara de carnaval a la luz de las velas. |
<!-- end generated -->

### Palabras

<!-- generated:terms -->
| Clásico (MOO1) | Era |
|---|---|
| Planeta | Ciudad |
| Población | Pueblo |
| Fábricas | Talleres |
| Bases de misiles | Torres |
| Escudo planetario | Murallas |
| CB | oro |
| PI | de saber |
| Ecología | Tierra |
| Tecnología | Saber |
| Flota | Ejército en marcha |
| En órbita | Guarnición |
| Transportes | Migrantes y levas |
| Base colonial estándar | Campamento de colonos |
| Pequeño | Banda |
| Mediano | Compañía |
| Grande | Batallón |
| Enorme | Hueste |
| Nebulosa | Brumas |
| Portal estelar | Puerto del canal |
| Razas | Imperios |
| Tamaño de la galaxia | Tamaño del mundo |
| Red Galáctica de Noticias | El Heraldo |
| El Consejo Galáctico | El Gran Consejo |
| Orion | Persépolis |
| Guardián | Inmortales |
| Cristal Espacial | Compañía Catalana |
| Ameba Espacial | Gran Compañía |
<!-- end generated -->

### Tecnologías

El nivel es el lugar de la tecnología en su campo (1 a 50); el costo de investigación crece con él.

<!-- generated:techs -->
#### Computadoras / Erudición

| Nivel | Clásico (MOO1) | Era |
|---|---|---|
| 1 | Sensor de combate | Informes de exploradores |
| 2 | Interferidor ECM Mark I | Camuflaje I |
| 4 | Sensor de espacio profundo | Atalayas |
| 5 | Computadora de combate Mark II | Veteranos II |
| 7 | Interferidor ECM Mark II | Camuflaje II |
| 8 | Controles robóticos mejorados III | Talleres gremiales III |
| 10 | Computadora de combate Mark III | Veteranos III |
| 12 | Interferidor ECM Mark III | Camuflaje III |
| 13 | Sensor espacial mejorado | Almenaras |
| 15 | Computadora de combate Mark IV | Veteranos IV |
| 17 | Interferidor ECM Mark IV | Camuflaje IV |
| 18 | Controles robóticos mejorados IV | Talleres gremiales IV |
| 20 | Computadora de combate Mark V | Veteranos V |
| 22 | Interferidor ECM Mark V | Camuflaje V |
| 23 | Sensor espacial avanzado | Red de espías |
| 25 | Computadora de combate Mark VI | Veteranos VI |
| 27 | Interferidor ECM Mark VI | Camuflaje VI |
| 28 | Controles robóticos mejorados V | Talleres gremiales V |
| 30 | Computadora de combate Mark VII | Veteranos VII |
| 32 | Interferidor ECM Mark VII | Camuflaje VII |
| 34 | Comunicaciones hiperespaciales | Palomas mensajeras |
| 35 | Computadora de combate Mark VIII | Veteranos VIII |
| 37 | Interferidor ECM Mark VIII | Camuflaje VIII |
| 38 | Controles robóticos mejorados VI | Talleres gremiales VI |
| 40 | Computadora de combate Mark IX | Veteranos IX |
| 42 | Interferidor ECM Mark IX | Camuflaje IX |
| 45 | Computadora de combate Mark X | Veteranos X |
| 46 | Interfaz Oráculo | Tambores de señales |
| 47 | Interferidor ECM Mark X | Camuflaje X |
| 48 | Controles robóticos mejorados VII | Talleres gremiales VII |
| 49 | Anulador de tecnología | Capitanes sobornados |
| 50 | Computadora de combate Mark XI | Veteranos XI |

#### Construcción / Albañilería

| Nivel | Clásico (MOO1) | Era |
|---|---|---|
| 1 | Tanques de combustible de reserva | Carros de suministros |
| 3 | Tecnología industrial mejorada 9 | Maestros constructores 9 |
| 5 | Desechos industriales reducidos al 80% | Saneamiento 80% |
| 8 | Tecnología industrial mejorada 8 | Maestros constructores 8 |
| 10 | Blindaje de duraleación | Armadura de malla |
| 11 | Trajes de combate | Gambesones |
| 13 | Tecnología industrial mejorada 7 | Maestros constructores 7 |
| 14 | Sistema de reparación automática | Cirujanos de campaña |
| 15 | Desechos industriales reducidos al 60% | Saneamiento 60% |
| 17 | Blindaje de zortio | Armadura de escamas |
| 18 | Tecnología industrial mejorada 6 | Maestros constructores 6 |
| 23 | Tecnología industrial mejorada 5 | Maestros constructores 5 |
| 24 | Exoesqueleto blindado | Brigantina |
| 25 | Desechos industriales reducidos al 40% | Saneamiento 40% |
| 26 | Blindaje de andrio | Armadura de placas |
| 28 | Tecnología industrial mejorada 4 | Maestros constructores 4 |
| 33 | Tecnología industrial mejorada 3 | Maestros constructores 3 |
| 34 | Blindaje de tritanio | Armadura de placas de acero |
| 35 | Desechos industriales reducidos al 20% | Saneamiento 20% |
| 36 | Control de daños avanzado | Médicos reales |
| 38 | Tecnología industrial mejorada 2 | Maestros constructores 2 |
| 40 | Armadura motorizada | Armadura completa |
| 42 | Blindaje de adamantio | Armadura milanesa |
| 45 | Eliminación de desechos industriales | Talleres limpios |
| 50 | Blindaje de neutronio | Armadura gótica |

#### Campos de fuerza / Fortificación

| Nivel | Clásico (MOO1) | Era |
|---|---|---|
| 1 | Escudos deflectores clase I | Muro de escudos I |
| 4 | Escudos deflectores clase II | Muro de escudos II |
| 8 | Escudo deflector personal | Broqueles |
| 10 | Escudos deflectores clase III | Muro de escudos III |
| 12 | Escudo planetario clase V | Murallas V |
| 14 | Escudos deflectores clase IV | Muro de escudos IV |
| 16 | Rayo repulsor | Muros de picas |
| 20 | Escudos deflectores clase V | Muro de escudos V |
| 21 | Escudo personal de absorción | Escudos de torre |
| 22 | Escudo planetario clase X | Murallas X |
| 24 | Escudos deflectores clase VI | Muro de escudos VI |
| 27 | Dispositivo de camuflaje | Capas de camuflaje |
| 30 | Escudos deflectores clase VII | Muro de escudos VII |
| 31 | Escudo Zyro | Manteletes |
| 32 | Escudo planetario clase XV | Murallas XV |
| 34 | Escudos deflectores clase IX | Muro de escudos IX |
| 37 | Campo de estasis | Ollas de humo |
| 38 | Escudo personal de barrera | Grandes escudos |
| 40 | Escudos deflectores clase XI | Muro de escudos XI |
| 42 | Escudo planetario clase XX | Murallas XX |
| 43 | Generador de agujeros negros | Minas de zapadores |
| 44 | Escudos deflectores clase XIII | Muro de escudos XIII |
| 46 | Escudo relámpago | Testudo |
| 50 | Escudos deflectores clase XV | Muro de escudos XV |

#### Planetología / Agricultura

| Nivel | Clásico (MOO1) | Era |
|---|---|---|
| 1 | Restauración ecológica | Silvicultura |
| 2 | Terraformado mejorado +10 | Irrigación +10 |
| 3 | Ambiente yermo controlado | Asentamiento en colinas rocosas |
| 5 | Restauración ecológica mejorada | Silvicultura mejorada |
| 6 | Ambiente de tundra controlado | Asentamiento en la tundra |
| 8 | Terraformado mejorado +20 | Irrigación +20 |
| 9 | Ambiente muerto controlado | Asentamiento en tierras baldías |
| 10 | Esporas mortales | Ratas de la peste |
| 12 | Ambiente infernal controlado | Asentamiento volcánico |
| 13 | Restauración ecológica potenciada | Cuidado de los bosques |
| 14 | Terraformado mejorado +30 | Irrigación +30 |
| 15 | Ambiente tóxico controlado | Asentamiento en pantanos |
| 16 | Enriquecimiento del suelo | Rotación de cultivos |
| 17 | Antídoto contra biotoxinas | Remedios herbales |
| 18 | Ambiente radiactivo controlado | Asentamiento en páramos |
| 20 | Terraformado mejorado +40 | Irrigación +40 |
| 21 | Clonación | Partería |
| 22 | Terraformado atmosférico | Recuperación de tierras |
| 24 | Restauración ecológica avanzada | Bosques reales |
| 26 | Terraformado mejorado +50 | Irrigación +50 |
| 27 | Virus de la perdición | Peste negra |
| 30 | Enriquecimiento del suelo avanzado | Sistema de tres campos |
| 32 | Terraformado mejorado +60 | Irrigación +60 |
| 34 | Restauración ecológica completa | Bosques sagrados |
| 36 | Antídoto universal | Medicina |
| 38 | Terraformado mejorado +80 | Irrigación +80 |
| 40 | Bioexterminador | Pestilencia |
| 42 | Clonación avanzada | Hospitales |
| 44 | Terraformado mejorado +100 | Irrigación +100 |
| 50 | Terraformado completo | Grandes acueductos |

#### Propulsión / Caminos

| Nivel | Clásico (MOO1) | Era |
|---|---|---|
| 1 | Motores retro | Marcha |
| 3 | Celdas de combustible de hidrógeno | Carretas de suministros |
| 5 | Celdas de combustible de deuterio | Trenes de suministros |
| 6 | Motores nucleares | Cría de ponis |
| 9 | Celdas de combustible de iridio | Graneros |
| 10 | Estabilizador inercial | Instrucción de caballería ligera |
| 12 | Impulsores sublumínicos | Cría de caballos |
| 14 | Cristales de dotomita | Red de caminos |
| 16 | Púlsar de energía | Anillo de fuego |
| 18 | Impulsores de fusión | Cría de corceles |
| 19 | Celdas de combustible de uridio | Caminos empedrados |
| 20 | Disipador warp | Abrojos |
| 23 | Celdas de combustible de Reajax II | Caminos reales |
| 24 | Impulsores de impulso | Carros de guerra |
| 27 | Portales estelares intergalácticos | Grandes canales |
| 29 | Cristales de trilitio | Postas |
| 30 | Impulsores iónicos | Cría de caballos árabes |
| 34 | Foco de alta energía | Maestros arqueros |
| 36 | Impulsores de antimateria | Cría de corceles turcomanos |
| 38 | Teletransportador subespacial | Sendas secretas |
| 40 | Púlsar iónico | Tormenta de fuego |
| 41 | Celdas de torio | Correo imperial |
| 42 | Impulsores interfásicos | Galeras fluviales |
| 43 | Interdictor subespacial | Puestos de centinela |
| 45 | Transportadores de combate | Escalas de asedio |
| 46 | Anulador inercial | Instrucción de caballería |
| 48 | Hiperimpulsores | Postas de relevo |
| 50 | Dispositivo de desplazamiento | Estandartes señuelo |

#### Armas / Herrería

| Nivel | Clásico (MOO1) | Era |
|---|---|---|
| 1 | Láseres | Arquería |
| 2 | Láseres de mano | Ballestas de mano |
| 4 | Cohetes Hyper V | Hachas arrojadizas |
| 5 | Láser Gatling | Ballestas de repetición |
| 6 | Cohetes antimisiles | Paveses |
| 7 | Cañón de perdigones de neutrones | Hondas |
| 8 | Cohetes Hyper X | Dardos |
| 9 | Bomba de fusión | Fuego griego |
| 10 | Cañón iónico | Ballestas |
| 11 | Cohetes Scatter Pack V | Lluvia de flechas |
| 12 | Rifle iónico | Picas |
| 13 | Acelerador de masa | Balistas |
| 14 | Misiles de merculita | Flechas de fuego |
| 15 | Desintegrador de neutrones | Cañones de mano |
| 16 | Bomba de antimateria | Morteros de asedio |
| 17 | Rayo de gravitones | Balas encadenadas |
| 18 | Misiles Stinger | Jabalinas dentadas |
| 19 | Rayo duro | Arcabuces |
| 20 | Rayo de fusión | Mosquetes |
| 21 | Proyector de corriente iónica | Cal viva |
| 22 | Bomba Omega V | Bombardas |
| 23 | Torpedos de antimateria | Onagros |
| 24 | Rifle de fusión | Alabardas |
| 25 | Cañón megarrayo | Escorpiones |
| 26 | Fásor | Fusiles |
| 27 | Misiles Scatter Pack VII | Tormenta de flechas |
| 28 | Desintegrador automático | Cañones de andanada |
| 29 | Misiles Pulson | Mangoneles |
| 30 | Rayo de taquiones | Metralla |
| 31 | Fásor de mano | Pistolas |
| 32 | Autocañón Gauss | Órganos de fuego |
| 33 | Rayo de partículas | Cañones de campaña |
| 34 | Misiles Hercular | Trebuchets |
| 35 | Cañón de plasma | Cañones |
| 36 | Rayo de la muerte | Espejos ustorios |
| 37 | Disruptor | Grandes cañones |
| 38 | Fásor pulsante | Fusiles rápidos |
| 39 | Bomba de neutronio | Grandes bombardas |
| 40 | Torpedos infernales | Trebuchets de fuego |
| 41 | Misiles Zeon | Cohetes |
| 42 | Rifle de plasma | Carabinas |
| 43 | Torpedos de protones | Cohetes de asedio |
| 44 | Misiles Scatter Pack X | Andanadas de cohetes |
| 45 | Cañón de plasma trifocal | Cañones triples |
| 46 | Conversor estelar | Tren de asedio |
| 47 | Proyector de corriente de neutrones | Sifones de fuego |
| 48 | Dispositivo triturador | Catapultas trituradoras |
| 50 | Torpedos de plasma | Brulotes |
<!-- end generated -->

Las tablas se generan a partir de los archivos de texto del juego con `tools/gen_moo1_doc.py`.

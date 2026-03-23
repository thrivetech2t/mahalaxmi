# Sistema Federado de Orquestación de IA con Cadena de Custodia Criptográfica para Flujos de Trabajo de Identificación Biométrica Multimodal

**BioMetrics** · Patente Pendiente

| | |
|---|---|
| Filing Date | March 22, 2026 |
| Disclosure Date | March 22, 2026 |
| Inventor | Ami Hoepner Nunez |
| Organization | ThriveTech Services LLC |
| Location | West Palm Beach, Florida, USA |
| Contact | Ami.nunez@mahalaxmi.ai |
| Web | https://mahalaxmi.ai/biometrics |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback,biometrics) to submit corrections, translation notes, or technical comments.

---

## Aviso Legal

Se ha presentado ante la Oficina de Patentes y Marcas de los Estados Unidos una Solicitud de Patente Provisional de EE. UU. que cubre esta invención, de conformidad con 35 U.S.C. § 111(b). Este documento de divulgación pública establece un registro público de la fecha de la invención. La especificación completa está archivada en la USPTO. Todos los derechos reservados. El uso de esta arquitectura en cualquier producto o sistema comercial requiere una licencia de ThriveTech Services LLC.

---

## Resumen

Este documento describe un sistema y método novedosos para la orquestación federada de inteligencia artificial aplicada a flujos de trabajo de identificación biométrica multimodal. La invención aborda las limitaciones de los Sistemas Automatizados de Identificación Biométrica (ABIS) convencionales mediante la introducción de:

**1. Un Gestor Raíz de Federación** que descompone los eventos de inscripción biométrica en fragmentos de modalidad y delega el procesamiento a nodos de dominio con una profundidad de delegación máxima de uno (1) — una arquitectura acotada que garantiza una cadena de custodia trazable.

**2. Un ciclo de consenso Gestor-Trabajador** en cada nodo de dominio, donde agentes de IA trabajadores especializados producen aserciones de identidad estructuradas (no solo puntuaciones numéricas) que son resueltas por un algoritmo de consenso de quórum.

**3. Un mecanismo criptográfico de cadena de custodia** mediante registros WorkUnitReceipt firmados digitalmente emitidos en cada paso de orquestación, vinculados por identificadores padre en una cadena de auditoría de solo adición y verificable.

**4. Una capa de cumplimiento de políticas de ámbito jurisdiccional** que aplica restricciones de comportamiento inmutables a cada llamada de inferencia de IA dentro de un nodo — operando en la capa de inferencia de IA, no en la capa de aplicación, y por lo tanto no eludible por el código de aplicación.

**5. Un método de deduplicación federada** que realiza la deduplicación de identidad entre jurisdicciones intercambiando únicamente puntuaciones de confianza de coincidencia y recibos firmados — las plantillas biométricas sin procesar nunca cruzan los límites de los nodos.

---

## Descripción General de la Arquitectura

El sistema opera como una jerarquía de tres niveles. El Gestor Raíz de Federación (Profundidad 0) recibe eventos de inscripción biométrica, los descompone en fragmentos de modalidad y delega cada fragmento a un Nodo de Dominio. Cada Nodo de Dominio ejecuta un ciclo de consenso Gestor-Trabajador para la modalidad asignada (Rostro/FR, Huella dactilar, Iris/Palma, etc.). Los resultados de todos los Nodos de Dominio fluyen hacia una Capa de Integración y Consenso que aplica lógica de fusión de quórum, produce el Registro de Identidad final y sella la cadena criptográfica WorkUnitReceipt.

La profundidad máxima de delegación se impone arquitectónicamente en un nivel. Los nodos de dominio no pueden sub-delegar a nodos adicionales, lo que garantiza que los registros de auditoría de cadena de custodia permanezcan acotados y completamente trazables.

---

## 1. Profundidad de Delegación Acotada

El Gestor Raíz de Federación impone una profundidad de delegación máxima de 1. Los nodos de dominio pueden recibir fragmentos delegados pero no pueden sub-delegar a nodos adicionales. Esta restricción se impone arquitectónicamente, no es una opción de configuración. Su propósito es garantizar que los registros de auditoría de cadena de custodia sean trazables y acotados — un requisito crítico en contextos de aplicación de la ley y gestión de identidad regulada.

---

## 2. Consenso Gestor-Trabajador

La técnica anterior en sistemas biométricos multimodales utiliza fusión de puntuaciones estadísticas — promedios ponderados de puntuaciones de coincidencia numéricas de procesadores de modalidad independientes. Esta invención utiliza un enfoque fundamentalmente diferente: los agentes de IA especializados que operan como Trabajadores producen **aserciones de identidad estructuradas** que comprenden una puntuación de confianza, una decisión categórica (POSITIVE_ID / NEGATIVE_ID / INCONCLUSIVE / QUALITY_REJECT / ESCALATE) y una declaración de razonamiento en lenguaje natural. El agente Gestor aplica un **algoritmo de consenso de quórum** a estas aserciones, con umbrales configurables y escalada humana obligatoria para casos inciertos.

---

## 3. WorkUnitReceipt — Cadena de Custodia Criptográfica

Cada paso en el flujo de trabajo de orquestación emite un WorkUnitReceipt que contiene: un ID de recibo globalmente único; ID de recibo padre (enlace de cadena); ID de nodo, profundidad y código de jurisdicción; ID de operador/agente; ID de sujeto, modalidad y tipo de acción; puntuación de confianza y aserción de identidad; hash SHA-256 de los datos biométricos procesados en este paso; y una firma digital Ed25519 por la clave privada del nodo originador.

Los recibos son de solo adición y están vinculados por IDs padre formando una cadena criptográficamente verificable desde la captura inicial hasta la determinación final. Esto constituye un rastro de auditoría admisible ante los tribunales.

---

## 4. Cumplimiento de Políticas de Ámbito Jurisdiccional en la Capa de Inferencia de IA

Una capa de cumplimiento de políticas se interpone en cada llamada de inferencia de IA dentro de un nodo. Impone la política de referencia (inmutable, todos los nodos) — por ejemplo: sin inferencia de culpabilidad a partir de puntuaciones biométricas únicamente, cadena de custodia requerida, ID de operador obligatorio — y la política jurisdiccional (específica del nodo, establecida en el aprovisionamiento) — por ejemplo: requisitos de consentimiento para menores, límites de retención, umbrales de escalada derivados de la legislación aplicable.

Esta capa opera **por debajo de la capa de aplicación** — no puede ser desactivada, eludida ni anulada por el código de aplicación que se ejecuta dentro del nodo. El cumplimiento es una restricción de infraestructura, no una convención de software.

---

## 5. Deduplicación Federada Sin Transmisión de Datos Biométricos Sin Procesar

La deduplicación entre jurisdicciones se realiza de la siguiente manera: cada nodo de dominio ejecuta una búsqueda de deduplicación local contra su propia galería; cada nodo transmite únicamente la puntuación de confianza de coincidencia y un WorkUnitReceipt firmado al Gestor Raíz de Federación; las plantillas biométricas sin procesar, los derivados y las imágenes nunca cruzan los límites de los nodos; y el Gestor Raíz de Federación aplica consenso entre las puntuaciones de confianza recibidas.

Esto satisface los requisitos de soberanía de datos, las regulaciones de privacidad y los mandatos de minimización de datos que prohíben la transmisión entre jurisdicciones de datos biométricos sin procesar.

---

## Aplicabilidad

Esta invención es aplicable a, pero no limitada a:

- Sistemas de inscripción e identificación biométrica para fuerzas del orden
- Control fronterizo y gestión de identidad en inmigración
- Redes de identidad de justicia penal de múltiples organismos
- Programas de identidad gubernamentales y empresariales que requieren operación federada
- Cualquier sistema que requiera procesamiento biométrico multimodal con registros de auditoría criptográficos y aplicación de cumplimiento de ámbito jurisdiccional

---

## Distinción de Técnica Anterior

El siguiente cuadro resume cómo esta invención difiere de los enfoques existentes en el campo:

| Enfoque Existente | Esta Invención |
|---|---|
| Fusión de puntuaciones estadísticas (promedio ponderado de puntuaciones de modalidad) | Consenso de quórum de agentes de IA con aserciones estructuradas y razonamiento |
| Sin rastro de auditoría o solo registros en la capa de aplicación | Cadena WorkUnitReceipt criptográfica, solo adición, firmada con Ed25519 |
| ABIS centralizado único | Nodos federados con profundidad acotada e intercambio de confianza entre nodos |
| Cumplimiento como función de interfaz de usuario o indicador de configuración | Cumplimiento aplicado en la capa de inferencia de IA, no eludible |
| La deduplicación entre jurisdicciones requiere compartir datos sin procesar | Deduplicación mediante puntuaciones de confianza únicamente — no se transmiten datos sin procesar |

---

## Información de Presentación

Se ha presentado ante la USPTO una Solicitud de Patente Provisional de EE. UU. que cubre la especificación completa de esta invención, incluida la descripción detallada, figuras y reivindicaciones. La fecha de presentación establecida es el **22 de marzo de 2026**. Debe presentarse una solicitud no provisional dentro de los 12 meses para reclamar el beneficio de esta fecha de presentación provisional.

**Inventor:** Ami Hoepner Nunez
**Entidad:** ThriveTech Services LLC, West Palm Beach, Florida
**Correspondencia:** ThriveTech Services LLC, West Palm Beach, Florida
**Contacto:** Ami.nunez@mahalaxmi.ai

Esta divulgación se hace pública para establecer la fecha de técnica anterior y el registro público. Patente Pendiente.
© 2026 ThriveTech Services LLC. Todos los derechos reservados.

---

*ThriveTech Services LLC · Ami Hoepner Nunez · March 2026*

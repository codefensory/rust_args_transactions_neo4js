Rust Transaction with Neo4j
===

Este es un ejercicio de transacciones que funciona con la misma logica de **Bitcoin**. Utiliza Transacciones con Entradas y Salidas, Clave Publica/Privada y Firmas digitales, utilizando Neo4j como base de datos, dejando de lado la Blockchain para este ejercicio.

Este proyecto esta hecho para *aprender*, se esta aprendiendo sobre Rust, Neo4j, Transacciones de entradas y salidas, Firmas digitales y Pares de claves publica/privada con ECDSA, y diferentes tipos de hash (Sha256, Ripemd160)

Nota: Hay acciones un poco raras, esto se hace porque se tiene pensado realizar la firma de los outputs desde algun cliente y el sistema se esta realizando con eso en mente, esto para que la clave privada del usuario solo la tenga el y no la envie ni comparta a ningun otro lado.

___

Instalacion
===

Para asegurar el funcionamiento de este proyecto necesitaras lo siguiente:
   - Neo4j Desktop
   - Version editada de neo4rs que pueda convertir un Array de algun BoltType (Ver en los pull request de neo4rs y en Cargo.toml)

Debe iniciar una base de datos local con Neo4j Desktop y configurar sus credenciales en el archivo **<span>main.rs</span>**

Luego de clonar este repositorio puede compilarlo:

```bash
$ cargo build
```


Con esto listo debe ser capaz de utilizar el proyecto.

Uso
===

El uso de este ejercicio es mediante argumentos, los argumentos que acepta son los siguientes:

run create_account
---

Este comando te devuelve una cuenta con la siguiente informacion

   - **Private Key**: Clave privada de la cuenta
   - **ID**: Identificador de la cuenta, esto es como la direccion de tu wallet bitcoin, pero mas simple


run balance `<id>`
---

Funciona para devolver el balance que contiene una cuenta dentro de la base de datos.
   
   Esto devolvera `Balance is: <Your Money>`


run coinbase `<id>` `<amount>`
---

Es la funcion para generar dinero de la nada, funciona como el coinbase o recompenza que generan los mineros al minar un bloque. El coinbase de este proyecto solo genera una transaccion nueva con una salida, y el valor es la cantidad que tu le agreges.


run send `<from_private_key>` `<to_address>` `<amount>`
---

La funcion mas importante, enviar dinero desde una cuenta a otra. Esta funcion recibe la clave privada del emisor para poder asi, firmar la transaccion. 

Funciona de la siguiente manera:
   - Obtiene las salidas sin usar del emisor.
   - Verifica que la transaccion no ha sido editada mal intencionalmente (Eso es basico, solo un hash)
   - Verifica que el balance de las salidas obtenidas cumplan con el monto a enviar.
   - Crea nuevas entradas utilizando las salidas.
   - Verifica que las entradas pertenezcan al emisor.
   - Firma las entradas.
   - Verifica si la clave publica puede validar la firma.
   - Crea una transaccion y relaciona la salida a el receptor y si hay algun excedente crea otra salida con ese sobrante hacia el emisor


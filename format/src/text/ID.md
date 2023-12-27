# Notes on ID scoping and usage

## MODULE SCOPE

Module-scoped IDs may be used at any point in the file. These are tracked in a
map along with the module syntax element as it is created.

### Type IDs
Usages:
* Func TypeUse
* Import Func TypeUse

### Func IDs
Usages:
* Function Body: call, ref.func
* Export Func 
* Element List
* Start

### Table IDs
Usages:
* Function Body: call\_indirect, table.\*
* Export Table
* Elem Segments

### Memory IDs
Note: Always 0 in current spec

Usages:
* Export Mem
* Data Segments

### Global IDs
* Funcion Body: global.\*
* Export Global

### Elem IDs
* Function Body: elem.drop, table.init

### Data IDs
* Function Body: data.drop, mem.init

## FUNCTION SCOPE

Function scoped IDs are only referenced from within the scope of the function,
so the tracking of these is easier.

### Local IDs
Function Body: local.\*

## BLOCK SCOPE

Block scope identifiers are tracked only with the scope of a function. However,
labels are counted from the inside out, so the binding of a particular ID name
may change as block nesting gets deeper.

### Label IDs
Function Body: br, br_if, br_table


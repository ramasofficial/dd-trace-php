--TEST--
Exceptions are handled in the correct catch frame
--FILE--
<?php
use DDTrace\SpanData;

function handleException(Exception $e, $function) {
    echo 'Exception was handled by '. $function . '(): ';
    echo $e->getMessage() . PHP_EOL;
    return 'HANDLED';
}

function level0() {
    try {
        return level1();
    } catch (Exception $e) {
        return handleException($e, __FUNCTION__);
    }
}

function level1() {
    try {
        return level2();
    } catch (Exception $e) {
        return handleException($e, __FUNCTION__);
    }
}

function level2() {
    try {
        return level3();
    } catch (RuntimeException $e) {
        echo 'You should not see this ' . __FUNCTION__;
        return 'RuntimeException caught';
    }
}

function level3() {
    throw new Exception('Oops!');
    return 'You should not see this ' . __FUNCTION__;
}

DDTrace\trace_function('level0', function(SpanData $s, $a, $retval) {
    $s->name = 'level0';
    $s->resource = $retval;
});

DDTrace\trace_function('level1', function(SpanData $s, $a, $retval) {
    $s->name = 'level1';
    $s->resource = $retval;
});

DDTrace\trace_function('level2', function(SpanData $s, $a, $retval) {
    $s->name = 'level2';
    $s->resource = $retval;
});

DDTrace\trace_function('level3', function(SpanData $s, $a, $retval) {
    $s->name = 'level3';
    $s->resource = $retval;
});

echo level0() . PHP_EOL;

array_map(function($span) {
    echo 'Span: ' . $span['name'];
    if (isset($span['resource'])) {
        echo '-' . $span['resource'];
    }
    if (isset($span['meta']['error.msg'])) {
        echo ' (' . $span['meta']['error.msg'] . ')';
    }
    echo PHP_EOL;
}, dd_trace_serialize_closed_spans());
?>
--EXPECTF--
Exception was handled by level1(): Oops!
HANDLED
Span: level0-HANDLED
Span: level1-HANDLED
Span: level2-level2 (Uncaught Exception: Oops! in %s:%d)
Span: level3-level3 (Uncaught Exception: Oops! in %s:%d)

--TEST--
hook_method prehook is called with the correct scope (no override)
--FILE--
<?php

DDTrace\hook_method('BaseClass', 'speak',
    function ($This, $scope, $args) {
        echo "${scope}::speak hooked in BaseClass.\n";
    }
);

DDTrace\hook_method('ChildClass', 'speak',
    function ($This, $scope, $args) {
        echo "${scope}::speak hooked in ChildClass.\n";
    }
);

class BaseClass
{
    public static function speak()
    {
        echo "I have spoken.\n";
    }
}

final class ChildClass extends BaseClass
{
}

final class Orthogonal
{
    public static function run()
    {
        BaseClass::speak();
        ChildClass::speak();
    }
}

Orthogonal::run();
?>
--EXPECT--
BaseClass::speak hooked in BaseClass.
I have spoken.
ChildClass::speak hooked in ChildClass.
ChildClass::speak hooked in BaseClass.
I have spoken.

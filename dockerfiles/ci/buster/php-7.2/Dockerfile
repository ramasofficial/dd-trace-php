FROM datadog/dd-trace-ci:buster AS base

ARG phpVersion
ENV PHP_INSTALL_DIR_DEBUG_ZTS=${PHP_INSTALL_DIR}/debug-zts
ENV PHP_INSTALL_DIR_DEBUG_NTS=${PHP_INSTALL_DIR}/debug
ENV PHP_INSTALL_DIR_NTS=${PHP_INSTALL_DIR}/nts
ENV PHP_VERSION=${phpVersion}

FROM base as build
ARG phpTarGzUrl
ARG phpSha256Hash
COPY 0001-Fixed-incorrect-behavior-of-internal-memory-debugger.patch /home/circleci
RUN set -eux; \
    curl -fsSL -o /tmp/php.tar.gz "${phpTarGzUrl}"; \
    (echo "${phpSha256Hash} /tmp/php.tar.gz" | sha256sum -c -); \
    tar xf /tmp/php.tar.gz -C "${PHP_SRC_DIR}" --strip-components=1; \
    rm -f /tmp/php.tar.gz; \
    cd ${PHP_SRC_DIR}; \
    ./buildconf --force; \
    patch Zend/zend_alloc.c /home/circleci/0001-Fixed-incorrect-behavior-of-internal-memory-debugger.patch;
COPY configure.sh /home/circleci

FROM build as php-debug-zts
RUN set -eux; \
    mkdir -p /tmp/build-php && cd /tmp/build-php; \
    /home/circleci/configure.sh \
        --enable-debug \
        --enable-maintainer-zts \
        --prefix=${PHP_INSTALL_DIR_DEBUG_ZTS} \
        --with-config-file-path=${PHP_INSTALL_DIR_DEBUG_ZTS} \
        --with-config-file-scan-dir=${PHP_INSTALL_DIR_DEBUG_ZTS}/conf.d; \
    make -j "$((`nproc`+1))"; \
    make install; \
    # In two steps, because: You've configured multiple SAPIs to be built. You can build only one SAPI module plus CGI, CLI and FPM binaries at the same time. \
    sed -i 's/--enable-embed/--with-apxs2=\/usr\/bin\/apxs2/' config.nice; \
    ./config.nice; \
    make -j "$((`nproc`+1))"; \
    cp .libs/libphp7.so ${PHP_INSTALL_DIR_DEBUG_ZTS}/lib/apache2handler-libphp.so; \
    mkdir -p ${PHP_INSTALL_DIR_DEBUG_ZTS}/conf.d;

FROM build as php-debug
RUN set -eux; \
    mkdir -p /tmp/build-php && cd /tmp/build-php; \
    /home/circleci/configure.sh \
        --enable-debug \
        --prefix=${PHP_INSTALL_DIR_DEBUG_NTS} \
        --with-config-file-path=${PHP_INSTALL_DIR_DEBUG_NTS} \
        --with-config-file-scan-dir=${PHP_INSTALL_DIR_DEBUG_NTS}/conf.d; \
    make -j "$((`nproc`+1))"; \
    make install; \
    # In two steps, because: You've configured multiple SAPIs to be built. You can build only one SAPI module plus CGI, CLI and FPM binaries at the same time. \
    sed -i 's/--enable-embed/--with-apxs2=\/usr\/bin\/apxs2/' config.nice; \
    ./config.nice; \
    make -j "$((`nproc`+1))"; \
    cp .libs/libphp7.so ${PHP_INSTALL_DIR_DEBUG_NTS}/lib/apache2handler-libphp.so; \
    mkdir -p ${PHP_INSTALL_DIR_DEBUG_NTS}/conf.d;

FROM build as php-nts
RUN set -eux; \
    mkdir -p /tmp/build-php && cd /tmp/build-php; \
    /home/circleci/configure.sh \
        --prefix=${PHP_INSTALL_DIR_NTS} \
        --with-config-file-path=${PHP_INSTALL_DIR_NTS} \
        --with-config-file-scan-dir=${PHP_INSTALL_DIR_NTS}/conf.d; \
    make -j "$((`nproc`+1))"; \
    make install; \
    # In two steps, because: You've configured multiple SAPIs to be built. You can build only one SAPI module plus CGI, CLI and FPM binaries at the same time. \
    sed -i 's/--enable-embed/--with-apxs2=\/usr\/bin\/apxs2/' config.nice; \
    ./config.nice; \
    make -j "$((`nproc`+1))"; \
    cp .libs/libphp7.so ${PHP_INSTALL_DIR_NTS}/lib/apache2handler-libphp.so; \
    mkdir -p ${PHP_INSTALL_DIR_NTS}/conf.d;

FROM base as final
COPY --chown=circleci:circleci --from=build $PHP_SRC_DIR $PHP_SRC_DIR
COPY --chown=circleci:circleci --from=php-debug-zts $PHP_INSTALL_DIR_DEBUG_ZTS $PHP_INSTALL_DIR_DEBUG_ZTS
COPY --chown=circleci:circleci --from=php-debug $PHP_INSTALL_DIR_DEBUG_NTS $PHP_INSTALL_DIR_DEBUG_NTS
COPY --chown=circleci:circleci --from=php-nts $PHP_INSTALL_DIR_NTS $PHP_INSTALL_DIR_NTS

RUN set -eux; \
    for phpVer in $(ls ${PHP_INSTALL_DIR}); \
    do \
        echo "Install exts for PHP $phpVer..."; \
        switch-php ${phpVer}; \
        pecl channel-update pecl.php.net; \
        iniDir=$(php -i | awk -F"=> " '/Scan this dir for additional .ini files/ {print $2}'); \
        \
        yes '' | pecl install apcu; echo "extension=apcu.so" >> ${iniDir}/apcu.ini; \
        yes '' | pecl install mcrypt; echo "extension=mcrypt.so" >> ${iniDir}/mcrypt.ini; \
        pecl install ast; echo "extension=ast.so" >> ${iniDir}/ast.ini; \
        yes 'no' | pecl install memcached; echo "extension=memcached.so" >> ${iniDir}/memcached.ini; \
        pecl install mongodb; echo "extension=mongodb.so" >> ${iniDir}/mongodb.ini; \
        pecl install redis; echo "extension=redis.so" >> ${iniDir}/redis.ini; \
        # Xdebug is disabled by default
        # 2.9.2
        pecl install xdebug-2.9.2; \
        cd $(php-config --extension-dir); \
        mv xdebug.so xdebug-2.9.2.so; \
        # 2.9.5
        pecl install xdebug-2.9.5; \
        cd $(php-config --extension-dir); \
        mv xdebug.so xdebug-2.9.5.so; \
        echo "zend_extension=opcache.so" >> ${iniDir}/../php-apache2handler.ini; \
    done;

RUN set -eux; \
    # Enable the apache config \
    echo 'LoadModule php7_module /usr/lib/apache2/modules/libphp.so' | sudo tee /etc/apache2/mods-enabled/php.load; \
    # Set the default PHP version
    switch-php debug;

# Install Composer
COPY --from=composer:2.2 /usr/bin/composer /usr/local/bin/composer

COPY welcome /etc/motd

CMD ["php-fpm", "-F"]

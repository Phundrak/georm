;;; Directory Local Variables            -*- no-byte-compile: t -*-
;;; For more information see (info "(emacs) Directory Variables")

((rustic-mode . ((fill-column . 80)))
 (sql-mode . ((eval . (progn
                        (setq-local lsp-sqls-connections
                                    `(((driver . "postgresql")
                                       (dataSourceName \,
                                                       (format "host=%s port=%s user=%s password=%s dbname=%s sslmode=disable"
                                                               (getenv "DB_HOST")
                                                               (getenv "DB_PORT")
                                                               (getenv "DB_USER")
                                                               (getenv "DB_PASSWORD")
                                                               (getenv "DB_NAME")))))))))))

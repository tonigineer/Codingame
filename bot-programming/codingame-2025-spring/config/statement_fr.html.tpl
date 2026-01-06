<!-- LEAGUES level1 level2 level3 level4 level5 -->

<div id="statement_back" class="statement_back" style="display: none"></div>
<div class="statement-body">
  <!-- LEAGUE ALERT -->
  <div
    style="
      color: #7cc576;
      background-color: rgba(124, 197, 118, 0.1);
      padding: 20px;
      margin-right: 15px;
      margin-left: 15px;
      margin-bottom: 10px;
      text-align: left;
    "
  >
    <div style="text-align: center; margin-bottom: 6px">
      <img
        src="//cdn.codingame.com/smash-the-code/statement/league_wood_04.png"
      />
    </div>
    <p style="text-align: center; font-weight: 700; margin-bottom: 6px">
      Ce challenge est basé sur un système de <b>ligues</b>.
    </p>
    <span class="statement-league-alert-content">
      Pour ce défi, plusieurs ligues pour le même jeu sont disponibles. Une fois
      vos compétences prouvées contre le premier Boss, vous accéderez à une
      ligue supérieure et des règles supplémentaires seront disponibles.
      <br /><br />
      <b>NOUVEAU :</b> Dans les ligues bois, votre soumission afrontera
      uniquement le boss. Vous devrez atteindre l'objectif spécifique a la ligue
      au moins 3 fois sur 5 pour etre promu.
    </span>
  </div>

  <!-- GOAL -->
  <div class="statement-section statement-goal">
    <h2>
      <span class="icon icon-goal">&nbsp;</span>
      <span>Objectif</span>
    </h2>
    <div class="statement-goal-content">
      <div>
        <!-- BEGIN level5 -->
        Remportez la bataille d’eau en contrôlant le plus de territoire, ou en
        éclaboussant suffisament votre adversaire&nbsp;!
        <!-- END -->
            <!-- BEGIN level1 -->
            Dans cette ligue, déplacez l’un de vos agents vers les
            coordonnées <const>(6,1)</const>, et l’autre vers
            <const>(6,3)</const>.
            <!-- END -->
            <!-- BEGIN level2 -->
            Dans cette ligue, 
            <strong>tirez chaque tour sur l’agent ennemi ayant le plus de
              <var>wetness</var> (trempage)</strong>
            avec chacun de vos agents.
            <!-- END -->

            <!-- BEGIN level3 -->
            <p>
              Dans cette ligue, mettez vos deux agents derrière la
              meilleure couverture et tirer sur l’ennemi opposé ayant la plus faible couverture.
            </p>
            <!-- END -->

            <!-- BEGIN level4 -->
            Dans cette ligue, éliminez les
            trois groupes composés uniquement d’agents ennemis avec votre
            réserve de bombes à eau.
            <!-- END -->
      </div>
    </div>
  </div>

  <!-- RULES -->
  <div class="statement-section statement-rules">
    <h2>
      <span class="icon icon-rules">&nbsp;</span>
      <span>Règles</span>
    </h2>

    <div class="statement-rules-content">
      <p>Le jeu se joue sur une <b>grille</b>.</p>
      <p>Chaque joueur contrôle une équipe d’<b>agents</b>.</p>

      <!-- BEGIN level5 -->
      <div class="full-statement-a">
        <p>
          À chaque tour, chaque agent peut effectuer une
          <strong>action de déplacement </strong> et/ou une
          <strong>action de combat</strong>.
        </p>
        <br />

        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🔥 Agents
        </h3>
        <p>
          Les agents sont les unités contrôlées par les joueurs sur le terrain.
          Ils possèdent des <b>attributs</b> et des actions.
        </p>

        <p>
          Chaque agent possède un compteur de <var>wetness</var> (trempage), qui
          augmente lorsqu’il est attaqué par des agents ennemis. Une fois que le
          <var>wetness</var> d’un agent atteint 100, il est éliminé et retiré de
          la partie.
        </p>

        <p>
          Les agents disposent également d’une valeur fixe de
          <var>soakingPower</var> (puissance) et d’<var>optimalRange</var>
          (portée). La puissance indique combien de <var>wetness</var> ils
          infligent normalement, tandis que la portée sert à appliquer une
          pénalité si la cible est trop éloignée.
        </p>

        <ul>
          <li>
            Jusqu’à <var>optimalRange</var>, le <b>tir</b> inflige 100% de leur
            <var>soakingPower</var>.
          </li>
          <li>
            Au-delà, et jusqu’à deux fois la <var>optimalRange</var>, le
            <b>tir</b> n’inflige que 50% de leur <var>soakingPower</var>.
          </li>
          <li>Au-delà, le tir échoue.</li>
        </ul>
        <p>
          <em
            >Remarque : toutes les distances sont calculées selon la formule de
            Manhattan.</em
          >
        </p>

        <p>
          Chaque agent a également un <var>shootCooldown</var>, qui représente
          le nombre de tours qu’il doit attendre après un <b>tir</b> avant de
          pouvoir utiliser l'action <action>SHOOT</action> à nouveau. Il peut
          quand même utiliser d’autres actions entre-temps.
        </p>

        <p>
          En plus du tir, chaque agent dispose d’un nombre limité de
          <var>splashBombs</var> (bombes à eau) qu’il peut lancer. Ce nombre est
          déterminé au début de la partie et peut différer selon les agents.
        </p>
        <br />
        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🎬 Actions
        </h3>

        <p>
          À chaque tour, vous devez donner une commande à chaque agent que vous
          contrôlez. Chaque commande peut inclure plusieurs actions ; au maximum
          une seule <b>action de déplacement</b> et une seule
          <b>action de combat</b>.<br /><br />
          Vous pouvez ordonner les actions dans l’ordre que vous souhaitez, mais
          l’ordre d’exécution dépendra de la priorité de chaque action ;
          consultez la section <strong>Ordre des actions par tour</strong> pour
          plus de détails.
        </p>

        <p>
          Le déplacement s’effectue avec la commande <action>MOVE x y</action>.
          Avec celle-ci, l’agent tentera de se déplacer vers la position x, y.
          Si la position cible n’est pas orthogonalement adjacente à l’agent,
          alors il tentera de s’en approcher en utilisant le chemin valide le
          plus court possible. Si l’action aboutit à un déplacement sur une case
          avec une couverture ou un autre agent dessus, le déplacement sera
          annulé. Si des agents entrent en collision en tentant de
          <action>MOVE</action>, leur déplacement sera annulé.
        </p>

        <p></p>

        <p>Il existe plusieurs actions de combat disponibles :</p>

        <ul>
          <li>
            <action>SHOOT id</action> : Tenter de tirer sur l’agent
            <var>agentId</var>. Cela infligera du <var>wetness</var> selon les
            valeurs de <var>optimalRange</var> et de <var>soakingPower</var> de
            l’agent, et sera réduit par toute réduction de dégâts dont bénéficie
            l’agent cible (voir l’action <action>HUNKER_DOWN</action> (se
            recroqueviller) et la section <strong>Couverture</strong>).
          </li>
          <li>
            <action>THROW x y</action> : Tenter de lancer une
            <b>bombe à eau</b> sur la position <var>x</var>, <var>y</var>. Les
            <b>bombes à eau</b> ne peuvent être lancées qu’à une distance
            maximale de <const>4</const> cases depuis l’agent. Elles infligent
            <const>30</const> de <var>wetness</var> aux agents sur la case visée
            et sur toutes les cases adjacentes (orthogonalement et
            diagonalement). Cette action <b>ignore</b> la réduction des dégâts
            fournie par les couvertures et le fait de se recroqueviller.
          </li>
          <li>
            <action>HUNKER_DOWN</action> : Se recroqueviller pour obtenir 25% de
            réduction de dégâts contre les tirs ennemis ce tour-ci. Cette action
            peut se cumuler avec les bonus de couverture (voir la section
            <strong>Couverture</strong> ci-dessous).
          </li>
        </ul>
        <p></p>

        <p>
          <em
            >Voir la section <strong>Protocole de jeu</strong> pour plus
            d’informations sur l’envoi de commandes à vos agents.</em
          >
        </p>
      </div>
      <!-- END -->

      <!-- BEGIN level4 -->
      <div class="tutorial-4">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objectif 4 : Lancer des bombes à eau
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=147595400748837"
            style="width: 60%; max-width: 300px"
          />
        </div>
        <p>
          Vos agents peuvent désormais courir et tirer derrière une
          <b>couverture</b> ! Dans cette nouvelle ligue, lancez des
          <strong>bombes à eau</strong> sur les ennemis pour infliger un énorme
          trempage <strong>quel que soit le type de couverture</strong>.
        </p>
        <bR />
        <p>
          Les agents commenceront parfois la partie avec un certain nombre de
          <b>bombes à eau</b>. Le nombre actuel de <b>bombes à eau</b> pour
          chaque agent est indiqué à chaque tour dans l’entrée standard avec la
          variable <var>splashBombs</var>.
        </p>
        <br />
        <p>
          Lancer une bombe à eau est une <strong>action de combat</strong>. Cela
          signifie que cette action peut être utilisée après une action
          <strong>MOVE</strong>, tout comme l’action <strong>SHOOT</strong>.
        </p>

        <br />

        <p>
          Un agent utilisant l’action <action>THROW x y</action> va tenter de
          lancer une bombe à eau sur la position <var>x</var>, <var>y</var>. Les
          <b>bombes à eau</b> ne peuvent être lancées qu’à une
          <b>distance maximale</b> de <const>4</const> cases depuis l’agent.
          Elles infligent 30 de <var>wetness</var> à la case où elles
          atterrissent, ainsi que 30 de <var>wetness</var> à toutes les cases
          adjacentes (orthogonalement et diagonalement).
        </p>
      </div>
      <!-- END -->

      <!-- BEGIN level3 -->
      <div class="tutorial-3-a">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objectif 3 : Se mettre à couvert
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=145554441949158"
            style="width: 60%; max-width: 300px"
          />
        </div>

        <p>
          Vos agents peuvent tirer sur les agents ennemis ! Dans cette nouvelle
          ligue, vous devrez gérer la <strong>couverture</strong>.
        </p>
      </div>
      <!-- END -->

      <!-- BEGIN level3 level5 -->
      <div class="cover-explanation">
        <br />
        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🛡️ Couverture
        </h3>
        <p>
          Chaque case de la <b>grille</b> est transmise à votre programme via
          l’entrée standard. Pour chaque colonne de chaque ligne, un
          <var>tileType</var> vous est donné. Il peut désormais avoir l’une des
          <b>trois</b> valeurs possibles :
        </p>
        <ul>
          <li><const>0</const> une case vide.</li>
        </ul>
        <ul>
          <li>
            <const>1</const> une case contenant une
            <strong>couverture basse.</strong>
          </li>
        </ul>
        <ul>
          <li>
            <const>2</const> une case contenant une
            <strong>couverture haute.</strong>
          </li>
        </ul>
        <p>
          Les cases avec une <strong>couverture</strong> sont infranchissables,
          et les agents contournent automatiquement ces cases lors d’une action
          <action>MOVE</action>.
        </p>
        <br />

        <figure>
          <img
            style="width: 60%; max-width: 80px; margin-bottom: 10px"
            src="https://static.codingame.com/servlet/fileservlet?id=145594581729355"
          />
        </figure>
        <p>
          Un agent qui bénéficie d'une couverture obtient une réduction des
          dégâts contre les tirs ennemis. Une couverture basse procure
          <const>50%</const> de protection, et une couverture haute procure
          <const>75%</const> de protection.
        </p>
        <p>
          <em
            >Par exemple, un agent à portée optimale et avec une puissance de
            trempage de <const>24</const> n’infligera que <const>6</const> de
            trempage à un ennemi derrière une couverture haute.</em
          >
        </p>
        <p>
          Pour bénéficier d’une couverture, l’agent doit être orthogonalement
          adjacent à celle-ci, et le tir ennemi doit venir du côté opposé à la
          case de couverture. La couverture est ignorée si les deux agents sont
          adjacents à la même couverture.
        </p>
        <p>
          Dans le cas où plusieurs couvertures peuvent s’appliquer, seule la
          couverture <b>la plus haute</b> compte.
        </p>
        <br />
        <p><strong>Exemples :</strong></p>
        <div class="statement-example-container">
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594221358130"
            />
            <div class="legend">
              <div class="description">
                Un agent adjacent à gauche d’une basse
                couverture. Tirer sur cet agent depuis une case verte entraînera
                une réduction des dégâts.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594162353521"
            />
            <div class="legend">
              <div class="description">
                Dans le cas où un tir peut être affecté par deux couvertures,
                seule la plus haute est prise en compte.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594244448576"
            />
            <div class="legend">
              <div class="description">
                Un agent qui n’est orthogonalement adjacent à aucune couverture,
                et ne bénéficie donc d’aucune réduction des dégâts.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594209782393"
            />
            <div class="legend">
              <div class="description">
                L’agent orange bénéficie d’une couverture basse alors que
                l’agent violet ne bénéficie d’aucune couverture.
              </div>
            </div>
          </div>
          <div class="statement-example statement-example-medium">
            <img
              src="https://static.codingame.com/servlet/fileservlet?id=145594180818284"
            />
            <div class="legend">
              <div class="description">
                Aucun de ces agents ne bénéficie d’une couverture l’un contre
                l’autre car ils sont tous deux adjacents à celle-ci.
              </div>
            </div>
          </div>
        </div>
      </div>
      <!-- END -->

      <!-- BEGIN level5 -->
      <div class="full-statement-b">
        <p>
          <em
            >Remarque : Se recroqueviller se cumule avec la couverture, ce qui
            signifie que la protection totale obtenue en se mettant à couvert et
            en se recroquevillant est de 75% ou 100%, pour une basse ou une
            couverture haute respectivement.</em
          >
        </p>
        <br />
        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          📈 Points
        </h3>
        <p>
          Vous gagnez des points en prenant le contrôle de cases lorsque vous
          contrôlez une zone plus grande que votre adversaire.
        </p>

        <p>
          Toute case qui est plus proche de l’un de vos agents que d’un agent
          ennemi est considérée comme étant sous votre contrôle. Cependant, si
          un agent a un <var>wetness</var> supérieure ou égale à
          <const>50</const>, la distance à cet agent sera
          <strong>doublée</strong> lors de cette comparaison.
        </p>

        <p>
          À chaque tour, Vous marquez autant de points que le nombre de cases
          <b>supplémentaires</b> que vous controlez par rapport à votre
          adversaire.
        </p>

        <br />

        <h3
          style="
            font-size: 16px;
            font-weight: 700;
            padding-top: 20px;
            color: #838891;
            padding-bottom: 15px;
          "
        >
          🎬 Ordre des actions pour un tour
        </h3>
        <p>
          Les tours de jeu sont synchrones, pour les deux joueurs et leurs
          agents (cela signifie que tous les agents exécutent leurs actions en
          même temps). Cependant, certaines actions ont priorité sur d’autres :
        </p>

        <ul>
          <li>Les actions <action>MOVE</action> passent en premier,</li>
          <li>Puis les actions <action>HUNKER_DOWN</action>,</li>
          <li>Puis <action>SHOOT</action> et <action>THROW</action>,</li>
          <li>Et enfin, le retrait de tout agent trempé.</li>
        </ul>
        <br />
      </div>
      <!-- END -->

      <!-- BEGIN level3 -->
      <div class="tutorial-3-b">
        <h2>Run &amp; Gun</h2>
        <p>
          À partir de cette ligue, vos agents peuvent effectuer
          <b>à la fois</b> une action <action>MOVE</action> et une action
          <action>SHOOT</action> lors du même tour. Séparez les deux actions par
          un point-virgule et l’agent assigné effectuera d’abord l’action
          <action>MOVE</action> puis tentera immédiatement de
          <action>SHOOT</action> depuis sa nouvelle position.
        </p>
        <p>
          Exemple :
          <action><code>1; MOVE 6 3; SHOOT 4</code></action>
        </p>
        <br />

        Dans cette ligue, vous aurez des agents de chaque côté de l’écran. Ils
        sont tous confrontés à deux agents ennemis à portée. Ils se trouvent
        également à <const>1</const> action <action>MOVE</action> de cases avec
        couverture. Pour battre cette ligue, vous devez déplacer vos deux agents
        derrière la couverture la plus haute disponible et leur faire tirer sur
        les ennemis à portée derrière la couverture la plus basse.
      </div>
      <!-- END -->

      <!-- BEGIN level2 -->
      <div class="tutorial-2">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objectif 2 : l’action <action>SHOOT</action>
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=145554402703235"
            style="width: 60%; max-width: 300px"
          />
        </div>

        <p>
          Vos agents peuvent se déplacer ! Dans cette nouvelle ligue, des
          <b>agents ennemis</b> sont entrés sur le terrain !
          <br />
          <br />Heureusement, vos agents peuvent aussi effectuer l’action
          <action>SHOOT</action>.
          <br />
          <br />Dans ce jeu, les agents peuvent se tirer dessus avec des
          <strong>pistolets à eau</strong>. Tirer sur un agent augmentera son
          <var>wetness</var>. Si le wetness (trempage) d’un agent atteint
          <const>100</const> ou plus, il est retiré de la partie.
        </p>
        <p>
          La quantité de <var>wetness</var> ajoutée à un agent lorsqu’il se fait
          tirer dessus est égale au <var>soakingPower</var> du tireur. Ceci peut
          aussi être appelé les <b>dégâts</b>.
        </p>
        <p>
          Cependant, cette quantité sera <strong>divisée par deux</strong> si la
          <b>distance de Manhattan</b> entre les deux agents est supérieure à
          l’<var>optimalRange</var> du tireur. Le tir <b>échouera</b> si la
          distance est supérieure à deux fois l’<var>optimalRange</var>, auquel
          cas aucun dégât n’est infligé.
        </p>
        <p></p>
        <p>
          Les agents ennemis seront présents dans la liste des agents dans
          l’entrée standard. Vous pouvez les identifier grâce à la variable
          <var>player</var>. Vous avez également accès a leur <var>agentId</var> et leur
          <var>wetness</var>. Les agents dont la variable <var>player</var> vaut <var>myId</var> sont les vôtres.
        </p>
        <p></p>
        <p>
          L’action <action>SHOOT id</action> indique à un agent de tirer sur
          l’agent ayant l’id indiqué. Chaque agent peut effectuer une action
          <action>SHOOT</action> par tour.
        </p>
        <br />
      </div>
      <!-- END -->

      <!-- BEGIN level1 -->
      <div class="tutorial-1">
        <h3
          style="
            font-size: 14px;
            font-weight: 700;
            padding-top: 5px;
            padding-bottom: 15px;
          "
        >
          Objectif 1 : l’action <action>MOVE</action>
        </h3>

        <div style="text-align: center; margin: 15px">
          <img
            src="https://static.codingame.com/servlet/fileservlet?id=145554460332080"
            style="width: 60%; max-width: 300px"
          />
        </div>

        <p>
          Chacun de vos agents occupe une case sur la grille. Deux agents ne
          peuvent pas occuper la même case. Chaque agent possède un
          <var>agentId</var> unique.
        </p>
        <p>
          Chaque agent peut effectuer une action <action>MOVE</action> par tour.
          En affichant une action <action> MOVE x y</action>
          sur la sortie standard, vous pouvez indiquer à un agent de se déplacer
          d’une case vers les coordonnées indiquées.
        </p>

        <p>
          Pour attribuer une action à un agent, affichez sur la sortie standard
          son <var>agentId</var> suivi de l’action désirée, les deux séparés par
          un <b>point-virgule</b>. <br /><br />
          Par exemple, la ligne suivante :<br />
          <action>1; MOVE 12 3</action><br />
          attribuera l’action <action>MOVE 12 3</action> à l’agent ayant
          <var>agentId</var> = 1.
        </p>

        <br />

        <p>
          Vous devez envoyer exactement <const>1</const> ligne par agent de
          votre équipe.
        </p>

        <p>
          <em>
            Les données de la grille et des agents sont fournies à votre
            programme via l’entrée standard. Plus de détails dans la section
            Protocole de jeu.
          </em>
        </p>
      </div>
      <!-- END -->

      <!-- Victory conditions -->
      <div class="statement-victory-conditions">
        <div class="icon victory"></div>
        <div class="blk">
          <div class="title">Conditions de victoire</div>

          <div class="text">
            <!-- BEGIN level1 -->
            Dans cette ligue, vous avez <const>deux</const> agents sur une
            petite grille.<br />
            Votre objectif est de déplacer l’un de vos agents vers les
            coordonnées <const>(6,1)</const>, et l’autre vers
            <const>(6,3)</const>.
            <!-- END -->
            <!-- BEGIN level2 -->
            Dans cette ligue, vous avez deux agents sur une petite grille. Votre
            objectif est de
            <strong
              >tirer chaque tour sur l’agent ennemi ayant le plus de
              trempage</strong
            >
            avec chacun de vos agents.
            <!-- END -->

            <!-- BEGIN level3 -->
            <p>
              Dans cette ligue, vous aurez exactement
              <strong>1 tour</strong> pour mettre vos à couvert vos deux agents, en les déplaçant vers la case adjacentes avec la meilleure couverture. Puis tirez sur l’un des deux ennemis d’en face, celui qui est le moins protégé par une couverture.
            </p>
            <!-- END -->

            <!-- BEGIN level4 -->
            Dans cette ligue, il y a quatre groupes d’agents retranchés, dont
            l’un comprend un de vos propres agents. Vous devez éliminer les
            trois groupes composés uniquement d’agents ennemis avec votre
            réserve limitée de bombes à eau. <b>Le tir est désactivé</b>.
            <!-- END -->
            <!-- BEGIN level5 -->
            <p>
              Le gagnant est le joueur qui remplit l’une des conditions
              suivantes :
            </p>

            <ul>
              <li>
                Atteindre <const>600</const> points <b>de plus</b> que son
                adversaire
              </li>
              <li>Éliminer tous les agents adverses</li>
              <li>
                Avoir le plus de points à la fin des <const>100</const>
                tours
              </li>
            </ul>
            <!-- END -->
          </div>
        </div>
      </div>

      <!-- Lose conditions -->
      <div class="statement-lose-conditions">
        <div class="icon lose"></div>
        <div class="blk">
          <div class="title">Conditions de défaite</div>
          <div class="text">
            <!-- BEGIN level1 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li><const>20 tours</const> se sont écoulés.</li>
              <li>
                Votre programme ne fournit pas de commande dans le temps imparti
                ou l’une des commandes est invalide.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level2 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li>
                Un ou plusieurs de vos agents ne tire pas sur l’adversaire le
                plus trempé.
              </li>
              <li>
                Votre programme ne fournit pas de commande dans le temps imparti
                ou l’une des commandes est invalide.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level3 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li>
                L’un de vos agents se déplace vers la mauvaise position ou ne
                tire pas sur la bonne cible.
              </li>
              <li>
                Votre programme ne fournit pas de commande dans le temps imparti
                ou l’une des commandes est invalide.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level4 -->
            <ul style="padding-top: 0; padding-bottom: 0">
              <li>Vous touchez l’un de vos propres agents.</li>
              <li><const>40 tours</const> se sont écoulés.</li>
              <li>
                Votre programme ne fournit pas de commande dans le temps imparti
                ou l’une des commandes est invalide.
              </li>
            </ul>
            <!-- END -->
            <!-- BEGIN level5 -->
            Votre programme ne fournit pas de commande dans le temps imparti ou
            l’une des commandes est invalide.
            <!-- END -->
          </div>
        </div>
      </div>

      <br />
      <h3
        style="
          font-size: 14px;
          font-weight: 700;
          padding-top: 5px;
          padding-bottom: 15px;
        "
      >
        🐞 Conseils de débogage
      </h3>
      <ul>
        <li>
          Survolez la grille pour voir des informations supplémentaires sur la
          case sous votre souris.
        </li>
        <li>
          Attribuez l’action spéciale <action>MESSAGE text</action> à un agent
          et ce texte apparaîtra au-dessus de votre agent.
        </li>
        <li>
          Appuyez sur l’icône d’engrenage du visualiseur pour accéder à des
          options d’affichage supplémentaires.
        </li>
        <li>
          Utilisez le clavier pour contrôler l’action : barre d’espace pour
          lire/mettre en pause, flèches pour avancer d’une image à la fois.
        </li>
      </ul>
    </div>
  </div>

  <!-- PROTOCOL -->

  <!-- BEGIN level1 level2 level3 level4 -->
  <details class="statement-section statement-protocol">
    <!-- END -->
    <!-- BEGIN level5 -->
    <details open class="statement-section statement-protocol">
      <!-- END -->

      <summary
        open
        style="cursor: pointer; margin-bottom: 10px; display: inline-block"
      >
        <span style="display: inline-block; margin-bottom: 10px"
          >Cliquez pour agrandir</span
        >
        <h2 style="margin-bottom: 0">
          <span class="icon icon-protocol">&nbsp;</span>
          <span>Protocole de jeu</span>
        </h2>
      </summary>

      <!-- Protocol block -->
      <div class="blk">
        <div class="title">Entrée d'initialisation</div>
        <div class="text">
          <span class="statement-lineno">Première ligne :</span> un entier
          <var>myId</var>, identifiant de votre joueur.
          <p>
            <span class="statement-lineno">Deuxième ligne :</span> un entier
            <var>agentDataCount</var> correspondant au nombre d'agents sur la
            grille.
          </p>
          <p>
            <span class="statement-lineno">Les </span><var>agentDataCount</var>
            <span class="statement-lineno">lignes suivantes :</span> Les
            <const>6</const> informations suivantes pour chaque agent :
          </p>
          <ul>
            <li><var>agentId</var> : identifiant unique de cet agent</li>
            <li>
              <var>player</var> : identifiant du joueur possédant cet agent
            </li>
            <li>
              <var>shootCooldown</var> : nombre minimal de tours entre deux tirs
              pour cet agent
            </li>
            <li>
              <var>optimalRange</var> : portée de tir optimale de cet agent
            </li>
            <li>
              <var>soakingPower</var> : dégâts de trempage maximaux que peut
              infliger cet agent
            </li>
            <li>
              <var>splashBombs</var> : nombre initial de bombes à eau
              disponibles pour cet agent
            </li>
          </ul>
          <p>
            <span class="statement-lineno">Ligne suivante :</span> deux entiers
            <var>width</var> et
            <var>height</var>
            pour la taille de la grille.
          </p>
          <p>
            <span class="statement-lineno">Les </span><var>width</var
            ><span class="statement-lineno"> * </span><var>height</var>
            <span class="statement-lineno">lignes suivantes :</span> Les 3
            informations suivantes pour chaque case de la grille :
          </p>
          <ul>
            <li><var>x</var> : coordonnée X (0 est tout à gauche)</li>
            <li><var>y</var> : coordonnée Y (0 est tout en haut)</li>
            <li>
              <var>tile_type</var> :
              <ul>
                <li><const>0</const> pour une case vide</li>
                <li><const>1</const> pour une couverture basse</li>
                <li><const>2</const> pour une couverture haute</li>
              </ul>
            </li>
          </ul>
        </div>
      </div>
      <div class="blk">
        <div class="title">Entrée pour un tour de jeu</div>
        <div class="text">
          <p>
            <span class="statement-lineno">Première ligne :</span> un entier
            <var>agentCount</var> correspondant au nombre d'agents restants sur
            la grille.
          </p>
          <p>
            <span class="statement-lineno">Les </span><var>agentCount</var>
            <span class="statement-lineno">lignes suivantes :</span> Les
            <const>6</const> informations suivantes pour chaque agent :
          </p>
          <ul>
            <li><var>agentId</var> : identifiant unique de cet agent</li>
            <li><var>x</var> : coordonnée X (0 est tout à gauche)</li>
            <li><var>y</var> : coordonnée Y (0 est tout en haut)</li>
            <li>
              <var>cooldown</var> : nombre de tours restant avant que cet agent
              puisse à nouveau tirer
            </li>
            <li>
              <var>splashBombs</var> : quantité actuelle de bombes à eau
              disponibles pour cet agent
            </li>
            <li><var>wetness</var> : trempage actuel de l’agent</li>
          </ul>
          <p>
            <span class="statement-lineno">Ligne suivante :</span> un entier
            <var>myAgentCount</var> correspondant au nombre d'agents contrôlés
            par le joueur.
          </p>
        </div>
      </div>

      <!-- Protocol block -->
      <div class="blk">
        <div class="title">Sortie</div>
        <div class="text">
          <p>
            Une seule ligne par agent, précédée de son <var>agentId</var> et
            suivie de ses actions :
          </p>
          <em>Au maximum une action de déplacement :</em>
          <ul>
            <li>
              <action>MOVE x y</action> : Tente de se déplacer vers la position
              <var>x</var>, <var>y</var>.
            </li>
          </ul>
          <em>Au maximum une action de combat :</em>
          <ul>
            <li>
              <action>SHOOT id</action> : Tente de tirer sur l’agent
              <var>agentId</var>.
            </li>
            <li>
              <action>THROW</action> : Tente de lancer une bombe à eau à la
              position <var>x</var>, <var>y</var>.
            </li>
            <li>
              <action>HUNKER_DOWN</action> : Se recroqueviller pour bénéficier
              de 25% de réduction de dégâts contre les attaques ennemies ce
              tour.
            </li>
          </ul>
          <em>Au maximum une action de message :</em>
          <ul>
            <li>
              <action>MESSAGE text</action> : Affiche <var>text</var> dans le
              visualiseur. Utile pour le débogage.
            </li>
          </ul>
          <p>
            Les instructions sont séparées par des points-virgules. Par exemple,
            la ligne suivante :
          </p>
          <p>
            <action><code>3;MOVE 12 3;SHOOT 5</code></action>
          </p>
          <p>
            Cela indique à <strong>l’agent 3</strong> de
            <strong>se déplacer vers les coordonnées (12, 3)</strong> puis de
            <strong>tirer sur l’agent 5</strong>.
          </p>
          <p>
            <em
              ><strong>Remarque :</strong> Le <var>agentId</var> au début peut
              être omis. Dans ce cas, les actions sont attribuées aux agents
              dans l’ordre croissant de leur <var>agentId</var>.</em
            >
          </p>
        </div>
      </div>

      <div class="blk">
        <div class="title">Contraintes</div>
        <div class="text">
          Temps de réponse par tour ≤ <const>50</const> ms <br />Temps de
          réponse pour le premier tour ≤ <const>1000</const> ms
          <!-- BEGIN level5 -->
          <br />
          <const>12</const> &le; <var>width</var> &le; <const>20</const>
          <br />
          <const>6</const> &le; <var>height</var> &le; <const>10</const>
          <br />
          <const>6</const> &le; <var>agentDataCount</var> &le; <const>10</const>
          <!-- END -->
        </div>
      </div>
      <!-- BEGIN level2 level3 level4 -->
    </details>
    <!-- END -->
    <!-- BEGIN level 5 -->
  </details>
  <!-- END -->

  <!-- SHOW_SAVE_PDF_BUTTON -->
</div>
